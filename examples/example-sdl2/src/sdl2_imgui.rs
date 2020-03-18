use sdl2;

use imgui::sys as imgui_sys;

use std::sync::Arc;
use std::sync::Mutex;

use minimum::ImguiManager;
use imgui_sdl2::ImguiSdl2;
use sdl2::video::Window;
use sdl2::mouse::MouseState;

struct Sdl2ImguiManagerInner {
    imgui_sdl2: ImguiSdl2,
}

// For sdl2::mouse::Cursor, a member of imgui_sdl2::ImguiSdl2
unsafe impl Send for Sdl2ImguiManagerInner {}

//TODO: Investigate usage of channels/draw lists
#[derive(Clone)]
pub struct Sdl2ImguiManager {
    imgui_manager: ImguiManager,
    inner: Arc<Mutex<Sdl2ImguiManagerInner>>,
}

// Wraps imgui (and winit integration logic)
impl Sdl2ImguiManager {
    pub fn imgui_manager(&self) -> ImguiManager {
        self.imgui_manager.clone()
    }

    // imgui and winit platform are expected to be pre-configured
    pub fn new(
        mut imgui_context: imgui::Context,
        window: &Window,
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

        let imgui_sdl2 = ImguiSdl2::new(&mut imgui_context, window);
        let imgui_manager = ImguiManager::new(imgui_context);

        let inner = Sdl2ImguiManagerInner { imgui_sdl2 };

        Sdl2ImguiManager {
            imgui_manager,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    // Call when a winit event is received
    //TODO: Taking a lock per event sucks
    pub fn handle_event(
        &self,
        event: &sdl2::event::Event,
    ) {
        self.imgui_manager.with_context(|context| {
            let mut inner = self.inner.lock().unwrap();
            let inner = &mut *inner;
            inner.imgui_sdl2.handle_event(context, event);
        });
    }

    pub fn ignore_event(
        &self,
        event: &sdl2::event::Event,
    ) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.imgui_sdl2.ignore_event(event)
    }

    // Start a new frame
    pub fn begin_frame(
        &self,
        window: &Window,
        mouse_state: &MouseState,
    ) {
        self.imgui_manager.with_context(|context| {
            let mut inner = self.inner.lock().unwrap();
            let inner = &mut *inner;
            inner
                .imgui_sdl2
                .prepare_frame(context.io_mut(), window, mouse_state);
        });

        self.imgui_manager.begin_frame();
    }

    // Finishes the frame. Draw data becomes available via get_draw_data()
    pub fn render(
        &self,
        window: &Window,
    ) {
        self.imgui_manager.with_ui(|ui| {
            let mut inner = self.inner.lock().unwrap();
            let inner = &mut *inner;
            inner.imgui_sdl2.prepare_render(ui, window);
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
        self.imgui_manager.font_atlas_texture()
    }

    // Returns true if a frame has been started (and not ended)
    pub fn is_frame_started(&self) -> bool {
        self.imgui_manager.is_frame_started()
    }

    // Returns draw data (render must be called first to end the frame)
    pub fn draw_data(&self) -> Option<&imgui::DrawData> {
        self.imgui_manager.draw_data()
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

fn init_imgui(window: &Window) -> imgui::Context {
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

    let (win_w, win_h) = window.size();
    let (draw_w, draw_h) = window.drawable_size();

    let display_size = [win_w as f32, win_h as f32];
    let display_framebuffer_scale = (
        (draw_w as f32) / (win_w as f32),
        (draw_h as f32) / (win_h as f32),
    );

    // We only use integer DPI factors, because the UI can get very blurry
    let scale_factor = f32::max(display_framebuffer_scale.0, display_framebuffer_scale.1).round();
    let font_size = (16.0 * scale_factor) as f32;

    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!("../../fonts/mplus-1p-regular.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

    imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;

    imgui
}

pub fn init_imgui_manager(window: &Window) -> Sdl2ImguiManager {
    let mut imgui_context = init_imgui(&window);
    Sdl2ImguiManager::new(imgui_context, window)
}
