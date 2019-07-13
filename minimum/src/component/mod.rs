
mod slab_storage;
mod vec_storage;
mod registry;

use crate::slab;
use slab::RawSlabKey;
use slab::RawSlab;

use crate::entity;
use entity::EntityHandle;
use entity::EntitySet;

pub trait ComponentStorageBase {
    fn on_entity_free(&mut self, entity: &EntityHandle);
    fn has_component_for_entity(&self, entity: &EntityHandle);
}

//TODO: Make these take some sort of private index type to prevent someone from
// trying to fetch components directly (these are not checking generation.. it's assumed
// we are calling through the entity code, which does a gen check there
pub trait ComponentStorage<T> : Send + Sync {
    fn new() -> Self;
    fn allocate(&mut self, entity: &EntityHandle, data: T);
    fn free(&mut self, entity: &EntityHandle);
    fn free_if_exists(&mut self, entity: &EntityHandle);
    fn get(&self, entity: &EntityHandle) -> Option<&T>;
    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T>;
}

impl<T> ComponentStorageBase for T where T : ComponentStorage<T> {
    fn on_entity_free(&mut self, entity: &EntityHandle) {
        self.free(entity);
    }

    fn has_component_for_entity(&self, entity: &EntityHandle) {
        self.get(entity).is_some();
    }
}

pub trait Component: Sized + Send + Sync + 'static {
    type Storage: ComponentStorage<Self>;
}

pub use slab_storage::SlabComponentStorage;
pub use vec_storage::VecComponentStorage;
pub use registry::ComponentRegistry;
