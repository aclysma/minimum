use imgui::sys as imgui_sys;

// Inner state for ImguiManager, which will be protected by a Mutex. Mutex protection required since
// this object is Send but not Sync
struct Inner {
    context: imgui::Context,

    // Pointer to the font atlas. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed
    font_atlas_texture: *mut imgui::FontAtlasTexture<'static>,

    // Pointer to the current UI. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed, and a frame has started and not ended
    ui: Option<*mut imgui::Ui<'static>>,

    // Handles the integration between imgui and winit
    platform: imgui_winit_support::WinitPlatform,

    // These are all refreshed when frame is started
    want_capture_keyboard: bool,
    want_capture_mouse: bool,
    want_set_mouse_pos: bool,
    want_text_input: bool,
}

// Rust assumes pointers in Inner are not safe to send, so we need to explicitly impl that here
unsafe impl Send for Inner {}

//TODO: Investigate usage of channels/draw lists
pub struct ImguiManager {
    inner: std::sync::Mutex<Inner>,
}

// Wraps imgui (and winit integration logic)
// All functions that take the lock are mutable to ensure exclusive access and that it
// won't actually block
impl ImguiManager {
    // imgui and winit platform are expected to be pre-configured
    pub fn new(
        mut imgui_context: imgui::Context,
        platform: imgui_winit_support::WinitPlatform,
    ) -> Self {
        // Ensure font atlas is built and cache a pointer to it
        let font_atlas_texture = {
            let mut fonts = imgui_context.fonts();
            let font_atlas_texture = Box::new(fonts.build_rgba32_texture());

            // Remove the lifetime of the texture
            let font_atlas_texture: *mut imgui::FontAtlasTexture =
                Box::into_raw(font_atlas_texture);
            let font_atlas_texture: *mut imgui::FontAtlasTexture<'static> =
                unsafe { std::mem::transmute(font_atlas_texture) };
            font_atlas_texture
        };

        ImguiManager {
            inner: std::sync::Mutex::new(Inner {
                context: imgui_context,
                font_atlas_texture,
                ui: None,
                platform,
                want_capture_keyboard: false,
                want_capture_mouse: false,
                want_set_mouse_pos: false,
                want_text_input: false,
            }),
        }
    }

    // Call when a winit event is received
    //TODO: Taking a lock per event sucks
    pub fn handle_event<T>(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<T>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let inner = &mut *inner;
        let context = &mut inner.context;
        let platform = &mut inner.platform;
        platform.handle_event(context.io_mut(), &window, &event);
    }

    // Allows access to the context without caller needing to be aware of locking
    #[allow(dead_code)]
    pub fn with_context<F>(&mut self, f: F)
    where
        F: FnOnce(&mut imgui::Context),
    {
        let mut inner = self.inner.lock().unwrap();
        (f)(&mut inner.context);
    }

    // Allows access to the ui without the caller needing to be aware of locking. A frame must be started
    pub fn with_ui<F>(&mut self, f: F)
    where
        F: FnOnce(&mut imgui::Ui),
    {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            warn!("Tried to use imgui ui when a frame was not started");
            return;
        }

        if let Some(ui) = inner.ui {
            unsafe {
                (f)(&mut *ui);
            }
        }
    }

    // Get reference to the underlying font atlas. The ref will be valid as long as this object
    // is not destroyed
    pub fn font_atlas_texture(&mut self) -> &imgui::FontAtlasTexture {
        let inner = self.inner.lock().unwrap();
        assert!(!inner.font_atlas_texture.is_null());
        unsafe { &*inner.font_atlas_texture }
    }

    fn take_ui(inner: &mut Inner) -> Option<Box<imgui::Ui<'static>>> {
        let mut ui = None;
        std::mem::swap(&mut inner.ui, &mut ui);

        if let Some(ui) = ui {
            return Some(unsafe { Box::from_raw(ui) });
        }

        None
    }

    // Start a new frame
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let mut inner_mutex_guard = self.inner.lock().unwrap();
        let mut inner = &mut *inner_mutex_guard;

        // Drop the old Ui if it exists
        if inner.ui.is_some() {
            warn!("a frame is already in progress, starting a new one");
            ImguiManager::take_ui(&mut inner);
        }

        inner
            .platform
            .prepare_frame(inner.context.io_mut(), window)
            .unwrap();
        let ui = Box::new(inner.context.frame());

        inner.want_capture_keyboard = ui.io().want_capture_keyboard;
        inner.want_capture_mouse = ui.io().want_capture_mouse;
        inner.want_set_mouse_pos = ui.io().want_set_mouse_pos;
        inner.want_text_input = ui.io().want_text_input;

        // Remove the lifetime of the Ui
        let ui_ptr: *mut imgui::Ui = Box::into_raw(ui);
        let ui_ptr: *mut imgui::Ui<'static> = unsafe { std::mem::transmute(ui_ptr) };

        // Store it as a raw pointer
        inner.ui = Some(ui_ptr);
    }

    // Returns true if a frame has been started (and not ended)
    pub fn is_frame_started(&mut self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.ui.is_some()
    }

    // Finishes the frame. Draw data becomes available via get_draw_data()
    pub fn render(&mut self, window: &winit::window::Window) {
        let mut inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            warn!("render() was called but a frame was not started");
            return;
        }

        let ui = ImguiManager::take_ui(&mut inner);
        if let Some(ui) = ui {
            inner.platform.prepare_render(&ui, window);
            ui.render();
        } else {
            warn!("ui did not exist");
        }
    }

    // Returns draw data (render must be called first to end the frame)
    pub fn draw_data(&mut self) -> Option<&imgui::DrawData> {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_some() {
            warn!("get_draw_data() was called but a frame is in progress");
            return None;
        }

        let draw_data = unsafe { imgui_sys::igGetDrawData() };
        if draw_data.is_null() {
            warn!("no draw data available");
            return None;
        }

        let draw_data = unsafe { &*(draw_data as *mut imgui::DrawData) };
        Some(draw_data)
    }

    pub fn want_capture_keyboard(&self) -> bool {
        self.inner.lock().unwrap().want_capture_keyboard
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.inner.lock().unwrap().want_capture_mouse
    }

    pub fn want_set_mouse_pos(&self) -> bool {
        self.inner.lock().unwrap().want_set_mouse_pos
    }

    pub fn want_text_input(&self) -> bool {
        self.inner.lock().unwrap().want_text_input
    }
}

impl Drop for ImguiManager {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();

        let mut ui = None;
        std::mem::swap(&mut inner.ui, &mut ui);

        // Drop the UI call if it exists
        if let Some(ui) = ui {
            let _ui = unsafe { Box::from_raw(ui) };
        }

        // Drop the font atlas
        unsafe { Box::from_raw(inner.font_atlas_texture) };
    }
}