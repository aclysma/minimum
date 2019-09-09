//! This module implements support for deferred construction of entities and their components.
//!
//! For the time being, minimum will only be aware of `EntityFactory.` You can write your own factories
//! but minimum would not be aware of them, so you'd need to be responsible for triggering the creation
//! logic yourself.
use crate::ComponentCreator;
use crate::EntityRef;
use crate::EntitySet;
use crate::ResourceMap;

use std::collections::VecDeque;

/// Interface for any EntityPrototype, which allows for deferred construction of entities/components.
///
/// Construction is deferred until EntitySet::update is called.
pub trait EntityPrototype: Send + Sync {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef);
}

/// A factory for EntityPrototype. EntitySet takes care of calling flush_creates for you.
///
/// This class, and its integration with EntitySet, is a convenience for allowing deferred entity construction.
/// It is not necessary to use it or EntityPrototype.
pub struct EntityFactory {
    /// Entities that we should create
    prototypes: VecDeque<Box<dyn EntityPrototype>>,
}

impl EntityFactory {
    /// Creates an empty factory
    pub(crate) fn new() -> Self {
        EntityFactory {
            prototypes: VecDeque::new(),
        }
    }

    /// Enqueues an entity to create. This will occur when flush_creates is called.
    pub fn enqueue_create(&mut self, prototype: Box<dyn EntityPrototype>) {
        self.prototypes.push_back(prototype);
    }

    //TODO: Redesign this so that we batch-create all components/entities rather than do them one-by-one
    /// Creates all queue entities
    pub(crate) fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &mut EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        for p in self.prototypes.drain(..) {
            let entity = entity_set.allocate_get();
            p.create(resource_map, &entity);
        }
    }
}

/// Represents an entity that can be created and the components it should start out with
pub struct BasicEntityPrototype {
    /// Prototypes for the components to place on the entity
    components: Vec<Box<dyn ComponentCreator>>,
}

impl BasicEntityPrototype {
    pub fn new(components: Vec<Box<dyn ComponentCreator>>) -> Self {
        BasicEntityPrototype { components }
    }
}

impl EntityPrototype for BasicEntityPrototype {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef) {
        for c in &self.components {
            c.enqueue_create(resource_map, &entity.handle());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::BasicComponentFactory;
    use crate::component::BasicComponentPrototype;
    use crate::component::ComponentStorage;
    use crate::component::SlabComponentStorage;
    use crate::Component;
    use named_type::NamedType;

    #[derive(Clone, NamedType)]
    struct TestComponent1;
    impl Component for TestComponent1 {
        type Storage = SlabComponentStorage<Self>;
    }

    #[derive(Clone, NamedType)]
    struct TestComponent2;
    impl Component for TestComponent2 {
        type Storage = SlabComponentStorage<Self>;
    }

    #[test]
    fn test_entity_prototype() {
        let world = crate::WorldBuilder::new()
            .with_component(<TestComponent1 as Component>::Storage::new())
            .with_component(<TestComponent2 as Component>::Storage::new())
            .with_component_factory(BasicComponentFactory::<TestComponent1>::new())
            .with_component_factory(BasicComponentFactory::<TestComponent2>::new())
            .with_resource(EntityFactory::new())
            .build();

        let resource_map = world.resource_map;

        {
            let c1_prototype = BasicComponentPrototype::new(TestComponent1);
            let c2_prototype = BasicComponentPrototype::new(TestComponent2);

            let c_list: Vec<Box<dyn ComponentCreator>> =
                vec![Box::new(c1_prototype), Box::new(c2_prototype)];

            let e_prototype = BasicEntityPrototype::new(c_list);
            resource_map
                .fetch_mut::<EntityFactory>()
                .enqueue_create(Box::new(e_prototype));
            //e_prototype.enqueue_create(&resource_map);
        }

        resource_map
            .fetch_mut::<EntitySet>()
            .flush_creates(&resource_map);

        let entity_set = resource_map.fetch::<EntitySet>();
        let c1_storage = resource_map.fetch::<<TestComponent1 as Component>::Storage>();
        let c2_storage = resource_map.fetch::<<TestComponent2 as Component>::Storage>();
        for e in entity_set.iter() {
            c1_storage.get(&e.handle()).unwrap();
            c2_storage.get(&e.handle()).unwrap();
        }
    }
}
