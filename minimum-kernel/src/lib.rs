#[macro_use]
extern crate log;

mod asset_storage;
pub use asset_storage::GenericAssetStorage;

mod component_registry;
pub use component_registry::ComponentRegistryBuilder;
pub use component_registry::ComponentRegistry;

pub mod util;

pub mod prefab_cooking;

pub mod resources;
pub mod pipeline;
pub mod systems;
