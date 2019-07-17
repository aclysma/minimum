use crate::slab;
use slab::GenSlab;
use slab::GenSlabKey;

use crate::component;
use component::Component;
use component::ComponentRegistry;
use component::ComponentStorage;

use crate::systems;

pub type EntityHandle = GenSlabKey<Entity>;

#[derive(Debug)]
pub struct Entity {
    // This is an option, but it is inteded to always be valid. We need to allocate before
    // we can get the handle for the allocation
    handle: Option<EntityHandle>,
}

impl Entity {
    pub fn new() -> Self {
        Entity { handle: None }
    }

    pub fn handle(&self) -> EntityHandle {
        self.handle.clone().unwrap()
    }
}

//TODO: This is dangerous.. it's not enforcing the entity can't be removed
//TODO: Should I remove the entity ref?
pub struct EntityRef<'e> {
    _entity: &'e Entity, // this ref is just for borrow checking
    handle: EntityHandle,
}

impl<'e> EntityRef<'e> {
    pub fn new(entity: &'e Entity, handle: EntityHandle) -> Self {
        EntityRef {
            _entity: entity,
            handle,
        }
    }

    pub fn add_component<T: Component>(&self, storage: &mut T::Storage, data: T) {
        storage.allocate(&self.handle, data);
    }

    pub fn remove_component<T: Component>(&self, storage: &mut T::Storage) {
        storage.free(&self.handle);
    }

    pub fn get_component<'c, T: Component>(&self, storage: &'c T::Storage) -> Option<&'c T> {
        storage.get(&self.handle)
    }

    pub fn get_component_mut<'c, T: Component>(
        &self,
        storage: &'c mut T::Storage,
    ) -> Option<&'c mut T> {
        storage.get_mut(&self.handle)
    }
}

pub struct EntitySet {
    slab: GenSlab<Entity>,
    component_registry: ComponentRegistry,
    pending_deletes: Vec<EntityHandle>,
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            slab: GenSlab::new(),
            component_registry: ComponentRegistry::new(),
            pending_deletes: vec![],
        }
    }

    pub fn register_component_type<T: Component>(&mut self) {
        self.component_registry.register_component::<T>();
    }

    pub fn allocate(&mut self) -> EntityHandle {
        let handle = self.slab.allocate(Entity::new());
        self.slab.get_mut(&handle).unwrap().handle = Some(handle.clone());
        handle
    }

    pub fn allocate_get(&mut self) -> EntityRef {
        let handle = self.slab.allocate(Entity::new());
        let mut entity = self.slab.get_mut(&handle).unwrap();
        entity.handle = Some(handle.clone());
        EntityRef::new(entity, handle)
    }

    pub fn enqueue_free(&mut self, entity_handle: &EntityHandle) {
        self.pending_deletes.push(entity_handle.clone());
    }

    pub fn entity_count(&self) -> usize {
        self.slab.active_count()
    }

    pub fn get_entity_ref(&self, entity_handle: &EntityHandle) -> Option<EntityRef> {
        let handle = (*entity_handle).clone();
        let e = self.slab.get(entity_handle)?;
        Some(EntityRef::new(e, handle))
    }

    //    pub fn get_entity(&self, entity_handle: &EntityHandle) -> Option<&Entity> {
    //        self.slab.get(entity_handle)
    //    }
    //
    //    pub fn get_entity_mut(&mut self, entity_handle: &EntityHandle) -> Option<&mut Entity> {
    //        self.slab.get_mut(entity_handle)
    //    }

    pub fn flush_free(&mut self, world: &systems::World) {
        for pending_delete in &self.pending_deletes {
            self.slab.free(pending_delete);
        }

        self.component_registry
            .on_entities_free(world, self.pending_deletes.as_slice());
        self.pending_deletes.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.slab.iter()
    }

    pub fn upgrade_index_to_handle(&self, index: u32) -> EntityHandle {
        self.slab.upgrade_index_to_handle(index).unwrap()
    }
}

pub struct Iter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::VecComponentStorage;

    struct TestComponent {
        _value: i32,
    }

    impl TestComponent {
        fn new(value: i32) -> Self {
            TestComponent { _value: value }
        }
    }

    impl Component for TestComponent {
        type Storage = VecComponentStorage<Self>;
    }

    #[test]
    fn test_entity_count() {
        let mut world = systems::World::new();
        let mut entity_set = EntitySet::new();
        world.insert(<TestComponent as Component>::Storage::new());
        entity_set.register_component_type::<TestComponent>();

        let entity = entity_set.allocate();
        assert_eq!(entity_set.entity_count(), 1);
        entity_set.enqueue_free(&entity);
        assert_eq!(entity_set.entity_count(), 1);
        entity_set.flush_free(&world);
        assert_eq!(entity_set.entity_count(), 0);
    }
    //
    //    #[test]
    //    fn test_get_entity() {
    //        let mut world = systems::World::new();
    //        let mut entity_set = EntitySet::new();
    //        world.insert(<TestComponent as Component>::Storage::new());
    //        entity_set.register_component_type::<TestComponent>();
    //
    //        let entity_handle = entity_set.allocate();
    //        assert!(entity_set.get_entity(&entity_handle).is_some());
    //
    //        entity_set.enqueue_free(&entity_handle);
    //        assert!(entity_set.get_entity(&entity_handle).is_some());
    //        entity_set.flush_free(&world);
    //        assert!(entity_set.get_entity(&entity_handle).is_none());
    //    }
    //
    //    #[test]
    //    fn test_get_entity_mut() {
    //        let mut world = systems::World::new();
    //        let mut entity_set = EntitySet::new();
    //        world.insert(<TestComponent as Component>::Storage::new());
    //        entity_set.register_component_type::<TestComponent>();
    //
    //        let entity_handle = entity_set.allocate();
    //        assert!(entity_set.get_entity_mut(&entity_handle).is_some());
    //
    //        entity_set.enqueue_free(&entity_handle);
    //        assert!(entity_set.get_entity_mut(&entity_handle).is_some());
    //        entity_set.flush_free(&world);
    //        assert!(entity_set.get_entity_mut(&entity_handle).is_none());
    //    }

    #[test]
    fn test_destroy_entity_releases_components() {
        // Save on typing..
        type Storage = <self::TestComponent as Component>::Storage;

        let mut world = systems::World::new();
        let mut entity_set = EntitySet::new();
        world.insert(<TestComponent as Component>::Storage::new());
        entity_set.register_component_type::<TestComponent>();

        // Create an entity
        let entity_handle = entity_set.allocate();
        let entity = entity_set.get_entity_ref(&entity_handle).unwrap();

        // Add the component
        {
            let mut test_component_storage = world.fetch_mut::<Storage>();
            entity.add_component(&mut *test_component_storage, TestComponent::new(1));
        }

        // Ensure after we enqueue free and flush free, the component is released
        entity_set.enqueue_free(&entity_handle);
        assert!(world.fetch::<Storage>().get(&entity_handle).is_some());
        entity_set.flush_free(&world);
        assert!(world.fetch::<Storage>().get(&entity_handle).is_none());
    }

    #[test]
    fn test_add_get_remove_component() {
        // Save on typing..
        type Storage = <self::TestComponent as Component>::Storage;

        let mut world = systems::World::new();
        let mut entity_set = EntitySet::new();
        world.insert(<TestComponent as Component>::Storage::new());
        entity_set.register_component_type::<TestComponent>();

        // Create an entity
        let entity_handle = entity_set.allocate();
        let entity = entity_set.get_entity_ref(&entity_handle).unwrap();

        let mut test_component_storage = world.fetch_mut::<Storage>();

        // Fail to find the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_none());

        // Add the component
        entity.add_component(&mut *test_component_storage, TestComponent::new(1));

        // Succeed in finding the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_some());

        // Remove the component
        entity.remove_component::<TestComponent>(&mut test_component_storage);

        // Fail to find the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_none());
    }

    #[test]
    fn iterate_entities_with_components() {
        /*
        let mask = 0x00000000;
        mask |= component.get_index<A>();
        mask |= component.get_index<B>();
        mask |= component.get_index<C>();

        for entity in entities {
            if entity.component_mask & mask != 0 {
                let a = entity.get<A>();
                let b = entity.get<B>();
                let c = entity.get<C>();
            }
        }
        */
    }
}
