mod registry;
mod slab_storage;
mod vec_storage;
mod component_factory;

use crate::slab;
use slab::RawSlab;
use slab::RawSlabKey;

use crate::entity;
use entity::EntityHandle;

pub use registry::ComponentRegistry;
pub use registry::ComponentFreeHandler;
pub use slab_storage::SlabComponentStorage;
pub use vec_storage::VecComponentStorage;
pub use component_factory::ComponentPrototype;
pub use component_factory::ComponentFactory;
pub use component_factory::CloneComponentPrototype;
pub use component_factory::CloneComponentFactory;

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

pub type ReadComponent<T> = crate::systems::Read<<T as Component>::Storage>;
pub type WriteComponent<T> = crate::systems::Write<<T as Component>::Storage>;
pub type ReadComponentOption<T> = crate::systems::ReadOption<<T as Component>::Storage>;
pub type WriteComponentOption<T> = crate::systems::WriteOption<<T as Component>::Storage>;