// Used for getting name of type, but only available in nightly
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate named_type_derive;

//#[macro_use]
//extern crate minimum_derive;

pub mod component;
pub mod dispatch;
pub mod entity;
pub mod slab;
pub mod reflect;
pub mod resource;
pub mod util;
pub mod world;

pub use dispatch::DispatchControl;

pub use entity::Entity;
pub use entity::EntityFactory;
pub use entity::EntityHandle;
pub use entity::EntityPrototype;
pub use entity::EntitySet;
pub use entity::PendingDeleteComponent;

pub use component::CloneComponentFactory;
pub use component::CloneComponentPrototype;
pub use component::Component;
pub use component::ComponentFactory;
pub use component::ComponentPrototype;
pub use component::ComponentStorage;
pub use component::{ReadComponent, ReadComponentOption, WriteComponent, WriteComponentOption};

pub use resource::Read;
pub use resource::ReadOption;
pub use resource::Resource;
pub use resource::ResourceMap;
pub use resource::ResourceMapBuilder;
pub use resource::Write;
pub use resource::WriteOption;

pub use world::WorldBuilder;

#[cfg(feature = "async_support")]
pub use dispatch::async_dispatch;

pub use dispatch::simple_dispatch;

#[cfg(feature = "async_support")]
pub use dispatch::async_dispatch::MinimumDispatcher;
#[cfg(feature = "async_support")]
pub use dispatch::async_dispatch::Task;
#[cfg(feature = "async_support")]
pub use dispatch::async_dispatch::TaskContext;

#[cfg(not(feature = "async_support"))]
pub use dispatch::simple_dispatch::MinimumDispatcher;
#[cfg(not(feature = "async_support"))]
pub use dispatch::simple_dispatch::Task;

//pub use world::World;

/*
pub trait TypeNameMacro {
    fn type_name() -> &'static str;
}

#[derive(TypeName)]
struct MyStruct {

}
*/
/*
trait ReflectMacro {

}

#[derive(Reflect)]
struct MyStruct {

}
*/