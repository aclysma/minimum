mod registry;
mod slab_storage;
mod vec_storage;

use crate::slab;
use slab::RawSlab;
use slab::RawSlabKey;

use crate::entity;
use entity::EntityHandle;

pub use registry::ComponentRegistry;
pub use registry::ComponentFreeHandler;
pub use slab_storage::SlabComponentStorage;
pub use vec_storage::VecComponentStorage;

//TODO: Make these take some sort of private index type to prevent someone from
// trying to fetch components directly (these are not checking generation.. it's assumed
// we are calling through the entity code, which does a gen check there
pub trait ComponentStorage<T> : Send + Sync
where
    T: Component,
{
    fn allocate(&mut self, entity: &EntityHandle, data: T);
    fn free(&mut self, entity: &EntityHandle);
    fn free_if_exists(&mut self, entity: &EntityHandle);
    fn exists(&self, entity: &EntityHandle) -> bool;
    fn get(&self, entity: &EntityHandle) -> Option<&T>;
    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T>;
}

pub trait Component: Sized + Send + Sync + 'static {
    type Storage: ComponentStorage<Self>;
}