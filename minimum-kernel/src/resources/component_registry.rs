use crate::ComponentRegistry;
use std::ops::Deref;
use std::sync::Arc;

pub struct ComponentRegistryResource {
    component_registry: Arc<ComponentRegistry>
}

impl ComponentRegistryResource {
    pub fn new(component_registry: ComponentRegistry) -> Self {
        ComponentRegistryResource {
            component_registry: Arc::new(component_registry)
        }
    }
}

impl Deref for ComponentRegistryResource {
    type Target = ComponentRegistry;

    fn deref(&self) -> &Self::Target {
        &self.component_registry
    }
}