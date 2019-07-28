use super::EntityHandle;

use crate::component;
use component::Component;
use component::ComponentStorage;

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

    pub(super) fn set_handle(&mut self, handle: EntityHandle) {
        self.handle = Some(handle);
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
