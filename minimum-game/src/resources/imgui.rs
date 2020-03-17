use crate::ImguiManager;
use std::ops::{Deref, DerefMut};

pub struct ImguiResource {
    manager: ImguiManager,
}

impl ImguiResource {
    pub fn new(manager: ImguiManager) -> Self {
        ImguiResource { manager }
    }
}

impl Deref for ImguiResource {
    type Target = ImguiManager;

    fn deref(&self) -> &ImguiManager {
        &self.manager
    }
}

impl DerefMut for ImguiResource {
    fn deref_mut(&mut self) -> &mut ImguiManager {
        &mut self.manager
    }
}
