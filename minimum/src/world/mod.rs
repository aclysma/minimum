
use crate::resource::{
    Resource,
    ResourceMap
};

use crate::component::{Component, ComponentStorage, ComponentPrototype, ComponentFreeHandler, ComponentFactory, ComponentRegistry};

use crate::entity::{EntitySet, EntityFactory, PendingDeleteComponent};

pub struct WorldBuilder {
    resource_map: ResourceMap,
    default_component_registry: ComponentRegistry
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder {
            resource_map: ResourceMap::new(),
            default_component_registry: ComponentRegistry::new()
        }
    }

    pub fn with_resource<R>(mut self, r: R) -> Self
        where
            R: Resource,
    {
        self.resource_map.insert(r);
        self
    }

    //TODO: The storage/factory types here are rendundant and a user could possibly pass a component/storage that doesn't match
    //TODO: I'd rather not have the systems layer aware of entities/components.
    pub fn with_component<C: Component, S: ComponentStorage<C> + 'static>(
        mut self,
        component_storage: S,
    ) -> Self {
        self.resource_map.insert(component_storage);
        self.default_component_registry.register_component::<C>();
        self
    }

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

    pub fn with_component_factory<P: ComponentPrototype, F: ComponentFactory<P>>(
        mut self,
        component_factory: F,
    ) -> Self {
        self.resource_map.insert(component_factory);
        self.default_component_registry.register_component_factory::<P, F>();
        self
    }

    pub fn insert<R>(&mut self, r: R)
        where
            R: Resource,
    {
        self.resource_map.insert(r);
    }

    pub fn build(mut self) -> ResourceMap {
        self = self.with_resource(EntityFactory::new());
        self = self.with_component(<PendingDeleteComponent as Component>::Storage::new());

        let entity_set = EntitySet::new(self.default_component_registry);
        self.resource_map.insert(entity_set);

        self.resource_map
    }
}
