use skulpin::winit;
use skulpin_plugin_imgui::imgui;

use imgui::sys as imgui_sys;

use std::sync::Arc;
use std::sync::Mutex;

use minimum::ImguiManager;

// Inner state for ImguiManager, which will be protected by a Mutex. Mutex protection required since
// this object is Send but not Sync
struct ImguiPlatformManagerInner {
    // Handles the integration between imgui and winit
    platform: imgui_winit_support::WinitPlatform,
}

//TODO: Investigate usage of channels/draw lists
#[derive(Clone)]
pub struct ImguiPlatformManager {
    imgui: ImguiManager,
    inner: Arc<Mutex<ImguiPlatformManagerInner>>,
}

// Wraps imgui (and winit integration logic)
impl ImguiPlatformManager {
    // imgui and winit platform are expected to be pre-configured
    pub fn new(
        mut imgui_context: imgui::Context,
        platform: imgui_winit_support::WinitPlatform,
    ) -> Self {
        // Ensure font atlas is built and cache a pointer to it
        let font_atlas_texture = {
            let mut fonts = imgui_context.fonts();
            let font_atlas_texture = Box::new(fonts.build_rgba32_texture());
            log::info!("Building ImGui font atlas");

            // Remove the lifetime of the texture. (We're assuming we have ownership over it
            // now since imgui_context is being passed to us)
            let font_atlas_texture: *mut imgui::FontAtlasTexture =
                Box::into_raw(font_atlas_texture);
            let font_atlas_texture: *mut imgui::FontAtlasTexture<'static> =
                unsafe { std::mem::transmute(font_atlas_texture) };
            font_atlas_texture
        };

        let imgui = ImguiManager::new(imgui_context);

        ImguiPlatformManager {
            imgui,
            inner: Arc::new(Mutex::new(ImguiPlatformManagerInner {
                platform
            }))
        }
    }

    // Call when a winit event is received
    //TODO: Taking a lock per event sucks
    pub fn handle_event<T>(
        &self,
        window: &winit::window::Window,
        event: &winit::event::Event<T>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let inner = &mut *inner;
        let context = &mut inner.context;
        let platform = &mut inner.platform;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::ReceivedCharacter(ch),
                ..
            } if *ch == '\u{7f}' => {
                // Do not pass along a backspace character
                // This hack can be removed when https://github.com/Gekkio/imgui-rs/pull/253 is
                // implemented upstream and I switch to using it
            }
            _ => {
                platform.handle_event(context.io_mut(), &window, &event);
            }
        }
    }

    fn take_ui(manager: &mut ImguiManager) -> Option<Box<imgui::Ui<'static>>> {
        let mut ui = None;
        std::mem::swap(&mut manager.inner.ui, &mut ui);

        if let Some(ui) = ui {
            return Some(unsafe { Box::from_raw(ui) });
        }

        None
    }

    // Start a new frame
    pub fn begin_frame(
        &self,
        window: &winit::window::Window,
    ) {
        let mut imgui_inner_mutex_guard = self.imgui.inner.lock().unwrap();
        let mut imgui_inner = &mut *imgui_inner_mutex_guard;

        // Drop the old Ui if it exists
        if imgui_inner.ui.is_some() {
            log::warn!("a frame is already in progress, starting a new one");
            ImguiManager::take_ui(&mut imgui_inner);
        }

        imgui_inner
            .platform
            .prepare_frame(imgui_inner.context.io_mut(), window)
            .unwrap();
        let ui = Box::new(imgui_inner.context.frame());

        imgui_inner.want_capture_keyboard = ui.io().want_capture_keyboard;
        imgui_inner.want_capture_mouse = ui.io().want_capture_mouse;
        imgui_inner.want_set_mouse_pos = ui.io().want_set_mouse_pos;
        imgui_inner.want_text_input = ui.io().want_text_input;

        // Remove the lifetime of the Ui
        let ui_ptr: *mut imgui::Ui = Box::into_raw(ui);
        let ui_ptr: *mut imgui::Ui<'static> = unsafe { std::mem::transmute(ui_ptr) };

        // Store it as a raw pointer
        imgui_inner.ui = Some(ui_ptr);
    }

    // Finishes the frame. Draw data becomes available via get_draw_data()
    pub fn render(
        &self,
        window: &winit::window::Window,
    ) {
        let mut imgui_inner = self.inner.lock().unwrap();

        if imgui_inner.ui.is_none() {
            log::warn!("render() was called but a frame was not started");
            return;
        }

        let ui = ImguiManager::take_ui(&mut imgui_inner);
        if let Some(ui) = ui {
            imgui_inner.platform.prepare_render(&ui, window);
            ui.render();
        } else {
            log::warn!("ui did not exist");
        }
    }

    // Allows access to the context without caller needing to be aware of locking
    #[allow(dead_code)]
    pub fn with_context<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Context),
    {
        self.imgui.with_context(f);
    }

    // Allows access to the ui without the caller needing to be aware of locking. A frame must be started
    pub fn with_ui<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Ui),
    {
        self.imgui.with_ui(f);
    }

    // Get reference to the underlying font atlas. The ref will be valid as long as this object
    // is not destroyed
    pub fn font_atlas_texture(&self) -> &imgui::FontAtlasTexture {
        self.imgui.font_atlas_texture()
    }

    // Returns true if a frame has been started (and not ended)
    pub fn is_frame_started(&self) -> bool {
        self.imgui.is_frame_started()
    }

    // Returns draw data (render must be called first to end the frame)
    pub fn draw_data(&self) -> Option<&imgui::DrawData> {
        self.imgui.draw_data()
    }

    pub fn want_capture_keyboard(&self) -> bool {
        self.imgui.want_capture_keyboard()
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.imgui.want_capture_mouse()
    }

    pub fn want_set_mouse_pos(&self) -> bool {
        self.imgui.want_set_mouse_pos()
    }

    pub fn want_text_input(&self) -> bool {
        self.imgui.want_text_input()
    }
}



















fn init_imgui(window: &winit::window::Window) -> imgui::Context {
    use imgui::Context;

    let mut imgui = Context::create();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = 1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let scale_factor = window.scale_factor().round();
    let font_size = (16.0 * scale_factor) as f32;
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!("../../fonts/mplus-1p-regular.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

    imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;

    imgui
}

pub fn init_imgui_manager(window: &winit::window::Window) -> ImguiPlatformManager {
    let mut imgui_context = init_imgui(&window);
    let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    imgui_platform.attach_window(
        imgui_context.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Rounded,
    );

    ImguiPlatformManager::new(imgui_context, imgui_platform)
}
