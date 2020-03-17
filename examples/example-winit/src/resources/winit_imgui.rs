use std::ops::{Deref, DerefMut};
use crate::winit_imgui::WinitImguiManager;

// For now just wrap the input helper that skulpin provides
pub struct WinitImguiManagerResource {
    pub winit_imgui_manager: WinitImguiManager,
}

impl WinitImguiManagerResource {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new(imgui_platform_manager: WinitImguiManager) -> Self {
        WinitImguiManagerResource {
            winit_imgui_manager: imgui_platform_manager,
        }
    }
}

impl Deref for WinitImguiManagerResource {
    type Target = WinitImguiManager;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.winit_imgui_manager
    }
}

impl DerefMut for WinitImguiManagerResource {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit_imgui_manager
    }
}
