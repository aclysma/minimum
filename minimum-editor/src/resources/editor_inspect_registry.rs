use std::ops::{Deref, DerefMut};
use crate::inspect::EditorInspectRegistry;

// For now just wrap the input helper that skulpin provides
pub struct EditorInspectRegistryResource {
    registry: EditorInspectRegistry,
}

impl EditorInspectRegistryResource {
    pub fn new(registry: EditorInspectRegistry) -> Self {
        EditorInspectRegistryResource { registry }
    }

    pub fn registry(&self) -> &EditorInspectRegistry {
        &self.registry
    }
}

impl Deref for EditorInspectRegistryResource {
    type Target = EditorInspectRegistry;

    fn deref(&self) -> &Self::Target {
        self.registry()
    }
}
