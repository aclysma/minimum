use crate::Component;
use crate::ComponentCreator;
use crate::ComponentPrototype;
use crate::EntitySet;
use crate::ResourceMap;
use crate::{BasicComponentPrototype, EntityHandle, EntityRef};

use std::collections::VecDeque;
use std::sync::Mutex;

//
// Create entity with list of components
//
pub struct SimpleEntityPrototype {
    components: Vec<Box<dyn ComponentCreator>>,
}

impl SimpleEntityPrototype {
    pub fn new(components: Vec<Box<dyn ComponentCreator>>) -> Self {
        SimpleEntityPrototype { components }
    }
}

impl EntityPrototype for SimpleEntityPrototype {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef) {
        for c in &self.components {
            c.enqueue_create(resource_map, &entity.handle());
        }
    }
}

pub trait EntityPrototype: Send + Sync {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef);
}

//
// Entity factory
//
pub struct EntityFactory {
    prototypes: VecDeque<Box<dyn EntityPrototype>>,
}

impl EntityFactory {
    pub fn new() -> Self {
        EntityFactory {
            prototypes: VecDeque::new(),
        }
    }

    pub fn enqueue_create(&mut self, prototype: Box<dyn EntityPrototype>) {
        self.prototypes.push_back(prototype);
    }

    pub fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &mut EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        for p in self.prototypes.drain(..) {
            let entity = entity_set.allocate_get();
            p.create(resource_map, &entity);
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
        let resource_map = crate::WorldBuilder::new()
            .with_component(<TestComponent1 as Component>::Storage::new())
            .with_component(<TestComponent2 as Component>::Storage::new())
            .with_component_factory(BasicComponentFactory::<TestComponent1>::new())
            .with_component_factory(BasicComponentFactory::<TestComponent2>::new())
            .with_resource(EntityFactory::new())
            .build();

        {
            let c1_prototype = BasicComponentPrototype::new(TestComponent1);
            let c2_prototype = BasicComponentPrototype::new(TestComponent2);

            let c_list: Vec<Box<dyn ComponentCreator>> =
                vec![Box::new(c1_prototype), Box::new(c2_prototype)];

            let e_prototype = SimpleEntityPrototype::new(c_list);
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
