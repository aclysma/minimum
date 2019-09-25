#![cfg_attr(not(feature = "std"), no_std)]
// Used for getting name of type, but only available in nightly
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

extern crate no_std_compat as std;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate derivative;

pub mod component;
pub mod entity;
pub mod resource;
pub mod slab;
pub mod task;
pub mod util;
pub mod world;

pub use entity::BasicEntityPrototype;
pub use entity::Entity;
pub use entity::EntityFactory;
pub use entity::EntityHandle;
pub use entity::EntityPrototype;
pub use entity::EntityRef;
pub use entity::EntitySet;
pub use entity::PendingDeleteComponent;

pub use component::BasicComponentFactory;
pub use component::BasicComponentPrototype;
pub use component::Component;
pub use component::ComponentCreateQueueFlushListener;
pub use component::ComponentPrototypeDyn;
pub use component::ComponentFactory;
pub use component::ComponentPrototype;
pub use component::ComponentStorage;
pub use component::{ReadComponent, ReadComponentOption, WriteComponent, WriteComponentOption};

pub use resource::DataRequirement;
pub use resource::Read;
pub use resource::ReadOption;
pub use resource::Resource;
pub use resource::ResourceMap;
pub use resource::ResourceMapBuilder;
pub use resource::Write;
pub use resource::WriteOption;

pub use world::UpdateLoopSingleThreaded;
pub use world::World;
pub use world::WorldBuilder;

pub use task::DispatchControl;
pub use task::Phase;
pub use task::ReadAllTask;
pub use task::ReadAllTaskImpl;
pub use task::ResourceTask;
pub use task::ResourceTaskImpl;
pub use task::TaskConfig;
pub use task::TaskContextFlags;
pub use task::TaskDependencyList;
pub use task::TaskDependencyListBuilder;
pub use task::TaskFactory;
pub use task::TaskScheduleBuilderMultiThread;
pub use task::TaskScheduleBuilderSingleThread;
pub use task::TaskScheduleMultiThread;
pub use task::TaskScheduleSingleThread;
pub use task::WriteAllTask;
pub use task::WriteAllTaskImpl;

pub use util::TrustCell;
