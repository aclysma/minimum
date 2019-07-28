#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(feature = "async_support")]
pub mod async_dispatcher;

pub mod component;
pub mod entity;
pub mod slab;
pub mod systems;

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

pub use systems::Read;
pub use systems::ReadOption;
pub use systems::Resource;
pub use systems::World;
pub use systems::WorldBuilder;
pub use systems::Write;
pub use systems::WriteOption;

#[cfg(feature = "async_support")]
pub use systems::async_dispatch as dispatch;

#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch as dispatch;

#[cfg(feature = "async_support")]
pub use systems::async_dispatch::MinimumDispatcher;
#[cfg(feature = "async_support")]
pub use systems::async_dispatch::Task;

#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch::MinimumDispatcher;
#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch::Task;
