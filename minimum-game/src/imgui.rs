use std::sync::Arc;
use std::sync::Mutex;
use imgui::sys as imgui_sys;

pub use imgui;
use imgui::{DrawCmdParams, DrawCmd};

pub struct ImguiFontAtlas {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImguiFontAtlas {
    pub fn new(texture: &imgui::FontAtlasTexture) -> Self {
        ImguiFontAtlas {
            width: texture.width,
            height: texture.height,
            data: texture.data.to_vec(),
        }
    }
}

pub enum ImguiDrawCmd {
    Elements {
        count: usize,
        cmd_params: DrawCmdParams,
    },
    ResetRenderState,
    //RawCallback is not supported
}

impl From<imgui::DrawCmd> for ImguiDrawCmd {
    fn from(draw_cmd: DrawCmd) -> Self {
        match draw_cmd {
            imgui::DrawCmd::Elements { count, cmd_params } => {
                ImguiDrawCmd::Elements { count, cmd_params }
            }
            imgui::DrawCmd::ResetRenderState => ImguiDrawCmd::ResetRenderState,
            _ => unimplemented!(),
        }
    }
}

pub struct ImguiDrawList {
    vertex_buffer: Vec<imgui::DrawVert>,
    index_buffer: Vec<imgui::DrawIdx>,
    command_buffer: Vec<ImguiDrawCmd>,
}

impl ImguiDrawList {
    pub fn vertex_buffer(&self) -> &[imgui::DrawVert] {
        &self.vertex_buffer
    }
    pub fn index_buffer(&self) -> &[imgui::DrawIdx] {
        &self.index_buffer
    }
    pub fn commands(&self) -> &[ImguiDrawCmd] {
        &self.command_buffer
    }
}

pub struct ImguiDrawData {
    draw_lists: Vec<ImguiDrawList>,
    pub total_idx_count: i32,
    pub total_vtx_count: i32,
    pub display_pos: [f32; 2],
    pub display_size: [f32; 2],
    pub framebuffer_scale: [f32; 2],
}

impl ImguiDrawData {
    pub fn new(draw_data: &imgui::DrawData) -> Self {
        let draw_lists: Vec<_> = draw_data
            .draw_lists()
            .map(|draw_list| {
                let vertex_buffer: Vec<_> = draw_list.vtx_buffer().iter().copied().collect();
                let index_buffer: Vec<_> = draw_list.idx_buffer().iter().copied().collect();
                let command_buffer: Vec<_> = draw_list.commands().map(|x| x.into()).collect();

                ImguiDrawList {
                    vertex_buffer,
                    index_buffer,
                    command_buffer,
                }
            })
            .collect();

        ImguiDrawData {
            draw_lists,
            total_idx_count: draw_data.total_idx_count,
            total_vtx_count: draw_data.total_vtx_count,
            display_pos: draw_data.display_pos,
            display_size: draw_data.display_size,
            framebuffer_scale: draw_data.framebuffer_scale,
        }
    }

    pub fn draw_lists(&self) -> &[ImguiDrawList] {
        &self.draw_lists
    }
}

// Inner state for ImguiManager, which will be protected by a Mutex. Mutex protection required since
// this object is Send but not Sync
struct ImguiManagerInner {
    context: imgui::Context,

    // Pointer to the font atlas. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed
    font_atlas_texture: *mut imgui::FontAtlasTexture<'static>,

    // Pointer to the current UI. Assuming no direct calls to C imgui interface, this pointer is
    // valid as long as context is not destroyed, and a frame has started and not ended
    ui: Option<*mut imgui::Ui<'static>>,

    // These are all refreshed when frame is started
    want_capture_keyboard: bool,
    want_capture_mouse: bool,
    want_set_mouse_pos: bool,
    want_text_input: bool,
}

// Rust assumes pointers in Inner are not safe to send, so we need to explicitly impl that here
unsafe impl Send for ImguiManagerInner {}

impl Drop for ImguiManagerInner {
    fn drop(&mut self) {
        let mut ui = None;
        std::mem::swap(&mut self.ui, &mut ui);

        // Drop the UI call if it exists
        if let Some(ui) = ui {
            let _ui = unsafe { Box::from_raw(ui) };
        }

        // Drop the font atlas
        unsafe { Box::from_raw(self.font_atlas_texture) };
    }
}

//TODO: Investigate usage of channels/draw lists
#[derive(Clone)]
pub struct ImguiManager {
    inner: Arc<Mutex<ImguiManagerInner>>,
}

// Wraps imgui (and winit integration logic)
impl ImguiManager {
    // imgui and winit platform are expected to be pre-configured
    pub fn new(mut imgui_context: imgui::Context) -> Self {
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

        ImguiManager {
            inner: Arc::new(Mutex::new(ImguiManagerInner {
                context: imgui_context,
                font_atlas_texture,
                ui: None,
                want_capture_keyboard: false,
                want_capture_mouse: false,
                want_set_mouse_pos: false,
                want_text_input: false,
            })),
        }
    }

    // Start a new frame
    pub fn begin_frame(&self) {
        let mut inner_mutex_guard = self.inner.lock().unwrap();
        let inner = &mut *inner_mutex_guard;

        // Drop the old Ui if it exists
        if inner.ui.is_some() {
            log::warn!("a frame is already in progress, starting a new one");
            Self::take_ui(&mut *inner);
        }

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

    fn take_ui(inner: &mut ImguiManagerInner) -> Option<Box<imgui::Ui<'static>>> {
        let mut ui = None;
        std::mem::swap(&mut inner.ui, &mut ui);

        if let Some(ui) = ui {
            return Some(unsafe { Box::from_raw(ui) });
        }

        None
    }

    pub fn render(&self) {
        let mut inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            log::warn!("render() was called but a frame was not started");
            return;
        }

        let ui = ImguiManager::take_ui(&mut inner);
        if let Some(ui) = ui {
            ui.render();
        } else {
            log::warn!("ui did not exist");
        }
    }

    // Allows access to the context without caller needing to be aware of locking
    pub fn with_context<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Context),
    {
        let mut inner = self.inner.lock().unwrap();
        (f)(&mut inner.context);
    }

    // Allows access to the ui without the caller needing to be aware of locking. A frame must be started
    pub fn with_ui<F>(
        &self,
        f: F,
    ) where
        F: FnOnce(&mut imgui::Ui),
    {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_none() {
            log::warn!("Tried to use imgui ui when a frame was not started");
            return;
        }

        if let Some(ui) = inner.ui {
            unsafe {
                (f)(&mut *ui);
            }
        }
    }

    pub fn copy_font_atlas_texture(&self) -> Option<ImguiFontAtlas> {
        let inner = self.inner.lock().unwrap();

        if inner.font_atlas_texture.is_null() {
            None
        } else {
            unsafe { Some(ImguiFontAtlas::new(&*inner.font_atlas_texture)) }
        }
    }

    // Get reference to the underlying font atlas. The ref will be valid as long as this object
    // is not destroyed
    #[allow(unused_unsafe)]
    pub unsafe fn sys_font_atlas_texture(&self) -> Option<&imgui::FontAtlasTexture> {
        let inner = self.inner.lock().unwrap();

        if inner.font_atlas_texture.is_null() {
            None
        } else {
            unsafe { Some(&*inner.font_atlas_texture) }
        }
    }

    // Returns true if a frame has been started (and not ended)
    pub fn is_frame_started(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.ui.is_some()
    }

    pub fn copy_draw_data(&self) -> Option<ImguiDrawData> {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_some() {
            log::warn!("get_draw_data() was called but a frame is in progress");
            return None;
        }

        let draw_data = unsafe { imgui_sys::igGetDrawData() };
        if draw_data.is_null() {
            log::warn!("no draw data available");
            return None;
        }

        let draw_data = unsafe { &*(draw_data as *mut imgui::DrawData) };
        Some(ImguiDrawData::new(draw_data))
    }

    // Returns draw data (render must be called first to end the frame)
    #[allow(unused_unsafe)]
    pub unsafe fn sys_draw_data(&self) -> Option<&imgui::DrawData> {
        let inner = self.inner.lock().unwrap();

        if inner.ui.is_some() {
            log::warn!("get_draw_data() was called but a frame is in progress");
            return None;
        }

        let draw_data = unsafe { imgui_sys::igGetDrawData() };
        if draw_data.is_null() {
            log::warn!("no draw data available");
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
