use crate::EntityHandle;
use crate::EntitySet;
use crate::ResourceMap;
use std::collections::VecDeque;

use crate::component::ComponentPrototype;

//
// Create entity with list of components
//
pub struct EntityPrototype {
    components: Vec<Box<dyn ComponentPrototypeWrapper>>,
}

impl EntityPrototype {
    pub fn new(components: Vec<Box<dyn ComponentPrototypeWrapper>>) -> Self {
        EntityPrototype { components }
    }
}

//
// ComponentListEntityPrototype wants to hold a list of component prototypes, but this trait has an
// associated type and it's not currently possible to have a Box<dyn Trait> without specifying the
// associated type. This trait just
//
pub trait ComponentPrototypeWrapper: Sync + Send {
    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle);
}

impl<T> ComponentPrototypeWrapper for T
where
    T: ComponentPrototype + Sync + Send,
{
    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle) {
        <T as ComponentPrototype>::enqueue_create(&self, resource_map, entity_handle);
    }
}

//
// Entity factory
//
pub struct EntityFactory {
    prototypes: VecDeque<EntityPrototype>,
}

impl EntityFactory {
    pub fn new() -> Self {
        EntityFactory {
            prototypes: VecDeque::new(),
        }
    }

    pub fn enqueue_create(&mut self, prototype: EntityPrototype) {
        self.prototypes.push_back(prototype);
    }

    pub fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &mut EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        for p in self.prototypes.drain(..) {
            let entity_handle = entity_set.allocate();
            for c in p.components {
                c.enqueue_create(resource_map, &entity_handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{CloneComponentFactory, DefaultComponentReflector};
    use crate::component::CloneComponentPrototype;
    use crate::component::ComponentStorage;
    use crate::component::SlabComponentStorage;
    use crate::Component;

    #[derive(Clone, typename::TypeName)]
    struct TestComponent1;
    impl Component for TestComponent1 {
        type Storage = SlabComponentStorage<Self>;
        type Reflector = DefaultComponentReflector<Self>;
    }

    #[derive(Clone, typename::TypeName)]
    struct TestComponent2;
    impl Component for TestComponent2 {
        type Storage = SlabComponentStorage<Self>;
        type Reflector = DefaultComponentReflector<Self>;
    }

    #[test]
    fn test_entity_prototype() {
        let resource_map = crate::ResourceMapBuilder::new()
            .with_component(<TestComponent1 as Component>::Storage::new())
            .with_component(<TestComponent2 as Component>::Storage::new())
            .with_component_factory(CloneComponentFactory::<TestComponent1>::new())
            .with_component_factory(CloneComponentFactory::<TestComponent2>::new())
            .with_resource(EntityFactory::new())
            .build();

        {
            let c1_prototype = CloneComponentPrototype::new(TestComponent1);
            let c2_prototype = CloneComponentPrototype::new(TestComponent2);

            let c_list: Vec<Box<dyn ComponentPrototypeWrapper>> =
                vec![Box::new(c1_prototype), Box::new(c2_prototype)];

            let e_prototype = EntityPrototype::new(c_list);
            resource_map
                .fetch_mut::<EntityFactory>()
                .enqueue_create(e_prototype);
            //e_prototype.enqueue_create(&resource_map);
        }

        resource_map.fetch_mut::<EntitySet>().flush_creates(&resource_map);

        let entity_set = resource_map.fetch::<EntitySet>();
        let c1_storage = resource_map.fetch::<<TestComponent1 as Component>::Storage>();
        let c2_storage = resource_map.fetch::<<TestComponent2 as Component>::Storage>();
        for e in entity_set.iter() {
            c1_storage.get(&e.handle()).unwrap();
            c2_storage.get(&e.handle()).unwrap();
        }
    }
}
