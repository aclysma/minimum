mod component_factory;
mod registry;
mod slab_storage;
mod vec_storage;

use crate::slab;
use slab::RawSlab;
use slab::RawSlabKey;

use crate::entity;
use entity::EntityHandle;

pub use component_factory::ComponentFactory;
pub use component_factory::ComponentPrototype;
pub use component_factory::ComponentCreator;
pub use registry::ComponentFreeHandler;
pub use registry::ComponentRegistry;
pub use slab_storage::SlabComponentStorage;
pub use vec_storage::VecComponentStorage;
use std::marker::PhantomData;

// This is a trivial factory/prototype. It's usable, but non-trivial downstream projects should probably
// implement their own version of this so that cross-cutting concerns like persistence can be handled
mod basic;
pub use basic::BasicComponentFactory;
pub use basic::BasicComponentPrototype;

//TODO: Make these take some sort of private index type to prevent someone from
// trying to fetch components directly (these are not checking generation.. it's assumed
// we are calling through the entity code, which does a gen check there
pub trait ComponentStorage<T>: Send + Sync
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

//TODO: All the reflector stuff can be pulled into a top-level module
pub trait ComponentReflector: Send + Sync
{
    fn type_name() -> &'static str;
    //fn to_string() -> &'static str;
}

pub struct DefaultComponentReflector<T>
where
    T: Component
{
    phantom_data: PhantomData<T>
}

impl<T> ComponentReflector for DefaultComponentReflector<T>
where
    T: Component + named_type::NamedType
{
    fn type_name() -> &'static str {
        T::type_name()
    }

//    fn to_string() -> &'static str {
//        "".to_string()
//    }
}

pub trait Component: Sized + Send + Sync + 'static {
    type Storage: ComponentStorage<Self>;
    type Reflector: ComponentReflector;

    fn get_name(&self) -> &'static str {
        <Self::Reflector as ComponentReflector>::type_name()
    }
}

pub type ReadComponent<T> = crate::resource::Read<<T as Component>::Storage>;
pub type WriteComponent<T> = crate::resource::Write<<T as Component>::Storage>;
pub type ReadComponentOption<T> = crate::resource::ReadOption<<T as Component>::Storage>;
pub type WriteComponentOption<T> = crate::resource::WriteOption<<T as Component>::Storage>;
