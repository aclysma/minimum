use super::EntityHandle;

use crate::component;
use component::Component;
use component::ComponentStorage;
use crate::component::ComponentAllocateResult;

/// Represents a created entity. All data for entities is stored in components. Usually, you will work
/// with an EntityRef, not an entity directly.
#[derive(Debug)]
pub struct Entity {
    /// THe handle of this entity
    ///
    /// This is an option, but it is intended to always be valid. We need to allocate before
    /// we can get the handle for the allocation
    handle: Option<EntityHandle>,
}

impl Entity {
    /// Create a stub entity. MUST CALL `set_handle` IMMEDIATELY! (The handle won't be known until
    /// after allocation)
    pub(super) fn new() -> Self {
        Entity { handle: None }
    }

    /// Get the handle of this entity.
    ///
    /// (The entity handle could become stale, pointing at and entity
    /// that no longer exists, but this is safely detected at runtime.)
    pub fn handle(&self) -> EntityHandle {
        self.handle.clone().unwrap()
    }

    /// Set the handle of the entity. This should be handled internally and never called manually.
    pub(super) fn set_handle(&mut self, handle: EntityHandle) {
        self.handle = Some(handle);
    }
}

//TODO: This is dangerous.. it's not enforcing the entity can't be removed
//TODO: Should I remove the entity ref?
/// A reference to an entity, which can be used to add/remove/get components on that entity. Having
/// an EntityRef proves the entity exists.
pub struct EntityRef<'e> {
    /// This ref is only used as a hint to the borrow checker that the entity must live at least
    /// as long as the Ref structure
    _entity: &'e Entity,

    /// Handle of the entity we point to
    handle: EntityHandle,
}

impl<'e> EntityRef<'e> {
    /// Creates an entity ref
    pub(super) fn new(entity: &'e Entity) -> Self {
        EntityRef {
            _entity: entity,
            handle: entity.handle(),
        }
    }

    /// Get the handle of this entity.
    ///
    /// (The entity handle could become stale, pointing at and entity
    /// that no longer exists, but this is safely detected at runtime.)
    pub fn handle(&self) -> EntityHandle {
        self.handle.clone()
    }

    /// Add a component to the entity. This is an immediate operation. Component prototypes are the
    /// recommended way to add components to entities.
    pub fn add_component<T: Component>(&self, storage: &mut T::Storage, data: T) -> ComponentAllocateResult {
        storage.allocate(&self.handle, data)
    }

    // TODO: Cannot support this without calling free handler
    // Remove a component from the entity. Removal is immediate, but the free
    //    pub fn remove_component<T: Component>(&self, storage: &mut T::Storage) {
    //        storage.free(&self.handle);
    //    }

    /// Gets a component for the given entity
    pub fn get_component<'c, T: Component>(&self, storage: &'c T::Storage) -> Option<&'c T> {
        storage.get(&self.handle)
    }

    /// Gets a component for the given entity
    pub fn get_component_mut<'c, T: Component>(
        &self,
        storage: &'c mut T::Storage,
    ) -> Option<&'c mut T> {
        storage.get_mut(&self.handle)
    }
}
