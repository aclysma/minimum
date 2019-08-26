mod clone_component;
pub use clone_component::CloneComponentFactory;
pub use clone_component::CloneComponentPrototype;

pub mod inspect;

pub mod persist;

mod prototype;
pub use prototype::FrameworkComponentPrototype;
pub use prototype::FrameworkEntityPrototype;
