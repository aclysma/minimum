use std::ops::{Deref, DerefMut};
use crate::imgui_support::ImguiPlatformManager;

// For now just wrap the input helper that skulpin provides
pub struct ImguiPlatformResource {
    pub imgui_platform_manager: ImguiPlatformManager,
}

impl ImguiPlatformResource {
    /// Create a new TimeState. Default is not allowed because the current time affects the object
    #[allow(clippy::new_without_default)]
    pub fn new(imgui_platform_manager: ImguiPlatformManager) -> Self {
        ImguiPlatformResource { imgui_platform_manager }
    }
}

impl Deref for ImguiPlatformResource {
    type Target = ImguiPlatformManager;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.imgui_manager
    }
}

impl DerefMut for ImguiPlatformResource {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.imgui_manager
    }
}
