extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

#[macro_use]
extern crate mopa;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate strum_macros;

#[cfg(feature = "dim2")]
extern crate ncollide2d as ncollide;
#[cfg(feature = "dim3")]
extern crate ncollide3d as ncollide;

extern crate minimum_base as base;

mod clone_component;
pub use clone_component::CloneComponentFactory;
pub use clone_component::CloneComponentPrototype;

pub mod components;
pub mod resources;

#[cfg(feature = "editor")]
pub mod inspect;

pub mod persist;

#[cfg(feature = "editor")]
pub mod select;

mod prototype;
pub use prototype::FrameworkComponentPrototypeDyn;
pub use prototype::FrameworkComponentPrototype;
pub use prototype::FrameworkEntityPersistencePolicy;
pub use prototype::FrameworkEntityPrototype;
pub use prototype::FrameworkEntityPrototypeInner;

pub mod tasks;

#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount, Debug)]
pub enum PlayMode {
    // Represents the game being frozen for debug purposes
    System,

    // Represents the game being puased by the user (actual meaning of this is game-specific)
    Paused,

    // Normal simulation is running
    Playing,
}

//PLAYMODE_COUNT exists due to strum_macros::EnumCount
const PLAY_MODE_COUNT: usize = PLAYMODE_COUNT;

pub mod context_flags {
    // For pause status. Flags will be set based on if the game is in a certain playmode
    pub const PLAYMODE_SYSTEM: usize = 1;
    pub const PLAYMODE_PAUSED: usize = 2;
    pub const PLAYMODE_PLAYING: usize = 4;

    // For multiplayer games:
    // - Dedicated Server will only run Net_Server
    // - Pure client will only have Net_Client
    // - "Listen" client will have both
    // - Singleplayer will have both
    pub const AUTHORITY_SERVER: usize = 8;
    pub const AUTHORITY_CLIENT: usize = 16;
}
