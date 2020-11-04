use std::sync::Arc;
use std::sync::Mutex;

use minimum::ImguiManager;

// Inner state for ImguiManager, which will be protected by a Mutex. Mutex protection required since
// this object is Send but not Sync
struct WinitImguiManagerInner {
    // Handles the integration between imgui and winit
    platform: imgui_winit_support::WinitPlatform,
}

//TODO: Investigate usage of channels/draw lists
#[derive(Clone)]
pub struct WinitImguiManager {
    imgui_manager: ImguiManager,
    inner: Arc<Mutex<WinitImguiManagerInner>>,
}

// Wraps imgui (and winit integration logic)
impl WinitImguiManager {
    pub fn imgui_manager(&self) -> ImguiManager {
        self.imgui_manager.clone()
    }

    // imgui and winit platform are expected to be pre-configured
    pub fn new(
        mut imgui_context: imgui::Context,
        platform: imgui_winit_support::WinitPlatform,
    ) -> Self {
        // Ensure font atlas is built and cache a pointer to it
        let _font_atlas_texture = {
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

        let imgui_manager = ImguiManager::new(imgui_context);

        WinitImguiManager {
            imgui_manager,
            inner: Arc::new(Mutex::new(WinitImguiManagerInner { platform })),
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
        let platform = &mut inner.platform;

        self.imgui_manager.with_context(|context| {
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
        })
    }

    // Start a new frame
    pub fn begin_frame(
        &self,
        window: &winit::window::Window,
    ) {
        self.imgui_manager.with_context(|context| {
            let inner = self.inner.lock().unwrap();
            inner
                .platform
                .prepare_frame(context.io_mut(), window)
                .unwrap();
        });

        self.imgui_manager.begin_frame();
    }

    // Finishes the frame. Draw data becomes available via get_draw_data()
    pub fn render(
        &self,
        window: &winit::window::Window,
    ) {
        self.imgui_manager.with_ui(|ui| {
            let mut inner = self.inner.lock().unwrap();
            inner.platform.prepare_render(&ui, window);
        });

        self.imgui_manager.render();
    }

    // Allows access to the context without caller needing to be aware of locking
    #[allow(dead_code)]
    pub fn with_context<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Context),
    {
        self.imgui_manager.with_context(f);
    }

    // Allows access to the ui without the caller needing to be aware of locking. A frame must be started
    pub fn with_ui<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Ui),
    {
        self.imgui_manager.with_ui(f);
    }

    // Get reference to the underlying font atlas. The ref will be valid as long as this object
    // is not destroyed
    pub fn font_atlas_texture(&self) -> &imgui::FontAtlasTexture {
        unsafe { self.imgui_manager.sys_font_atlas_texture().unwrap() }
    }

    // Returns true if a frame has been started (and not ended)
    pub fn is_frame_started(&self) -> bool {
        self.imgui_manager.is_frame_started()
    }

    // Returns draw data (render must be called first to end the frame)
    pub fn draw_data(&self) -> Option<&imgui::DrawData> {
        unsafe { self.imgui_manager.sys_draw_data() }
    }

    pub fn want_capture_keyboard(&self) -> bool {
        self.imgui_manager.want_capture_keyboard()
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.imgui_manager.want_capture_mouse()
    }

    pub fn want_set_mouse_pos(&self) -> bool {
        self.imgui_manager.want_set_mouse_pos()
    }

    pub fn want_text_input(&self) -> bool {
        self.imgui_manager.want_text_input()
    }
}
