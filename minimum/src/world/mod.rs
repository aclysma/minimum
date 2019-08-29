//! Stitches together all components of minimum.
use crate::resource::{Resource, ResourceMap};

use crate::component::{
    Component, ComponentCreateQueueFlushListener, ComponentFreeHandler,
    ComponentRegistry, ComponentStorage,
};

use crate::entity::{EntityFactory, EntitySet, PendingDeleteComponent};

/// A builder for setting up a `World`
pub struct WorldBuilder {
    resource_map: ResourceMap,
    default_component_registry: ComponentRegistry,
}

impl WorldBuilder {
    /// Creates an empty world builder.. without resources and without any components registered.
    /// Internal-only resources/components will be set up as well.
    pub fn new() -> Self {
        WorldBuilder {
            resource_map: ResourceMap::new(),
            default_component_registry: ComponentRegistry::new(),
        }
    }

    // Add a resource to the map
    pub fn with_resource<R>(mut self, r: R) -> Self
    where
        R: Resource,
    {
        self.resource_map.insert(r);
        self
    }

    //TODO: The storage/factory types here are rendundant and a user could possibly pass a component/storage that doesn't match
    /// Add a component type
    pub fn with_component<C: Component, S: ComponentStorage<C> + 'static>(
        mut self,
        component_storage: S,
    ) -> Self {
        self.resource_map.insert(component_storage);
        self.default_component_registry.register_component::<C>();
        self
    }

    /// Add a component type, but set it up with a custom ComponentFreeHandler. This is an extension point for handling
    /// custom cleanup logic for components
    pub fn with_component_and_free_handler<
        C: Component,
        S: ComponentStorage<C> + 'static,
        F: ComponentFreeHandler<C> + 'static,
    >(
        mut self,
        component_storage: S,
    ) -> Self {
        self.resource_map.insert(component_storage);
        self.default_component_registry
            .register_component_with_free_handler::<C, F>();
        self
    }

    /// Adds a component factory. Multiple factories are allowed per component type.
    pub fn with_component_factory<F: ComponentCreateQueueFlushListener>(
        mut self,
        component_factory: F,
    ) -> Self {
        self.resource_map.insert(component_factory);
        self.default_component_registry
            .register_component_factory::<F>();
        self
    }

    /// Adds a resource type/instance
    pub fn insert_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        self.resource_map.insert(r);
    }

    /// Constructs a resource map with all minimum types properly set up
    pub fn build(mut self) -> ResourceMap {
        self = self.with_resource(EntityFactory::new());
        self = self.with_component(<PendingDeleteComponent as Component>::Storage::new());

        let entity_set = EntitySet::new(self.default_component_registry);
        self.resource_map.insert(entity_set);

        self.resource_map
    }
}
