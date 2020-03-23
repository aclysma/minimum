use std::ops::{Deref, DerefMut};
use crate::imgui::Sdl2ImguiManager;

// For now just wrap the input helper that skulpin provides
pub struct Sdl2ImguiManagerResource {
    pub sdl2_imgui_manager: Sdl2ImguiManager,
}

impl Sdl2ImguiManagerResource {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new(sdl2_imgui_manager: Sdl2ImguiManager) -> Self {
        Sdl2ImguiManagerResource { sdl2_imgui_manager }
    }
}

impl Deref for Sdl2ImguiManagerResource {
    type Target = Sdl2ImguiManager;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sdl2_imgui_manager
    }
}

impl DerefMut for Sdl2ImguiManagerResource {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sdl2_imgui_manager
    }
}
