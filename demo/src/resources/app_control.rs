use std::ops::{Deref, DerefMut};
use skulpin::app::AppControl;

// For now just wrap the input helper that skulpin provides
pub struct AppControlResource {
    pub app_control: AppControl,
}

impl AppControlResource {
    pub fn new(app_control: AppControl) -> Self {
        AppControlResource { app_control }
    }
}

impl Deref for AppControlResource {
    type Target = AppControl;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.app_control
    }
}

impl DerefMut for AppControlResource {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_control
    }
}
