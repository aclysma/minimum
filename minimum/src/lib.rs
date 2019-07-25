#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(feature = "async_support")]
pub mod async_dispatcher;

pub mod component;
pub mod entity;
pub mod slab;
pub mod systems;

pub use entity::EntityHandle;
pub use entity::EntitySet;
pub use entity::Entity;

pub use component::Component;
pub use component::ComponentStorage;
pub use component::{ReadComponent, WriteComponent, ReadComponentOption, WriteComponentOption};

pub use systems::World;
pub use systems::WorldBuilder;

#[cfg(feature = "async_support")]
pub use systems::async_dispatch as dispatch;

#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch as dispatch;

#[cfg(feature = "async_support")]
pub use systems::async_dispatch::Task;
#[cfg(feature = "async_support")]
pub use systems::async_dispatch::MinimumDispatcher;

#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch::Task;
#[cfg(not(feature = "async_support"))]
pub use systems::simple_dispatch::MinimumDispatcher;

