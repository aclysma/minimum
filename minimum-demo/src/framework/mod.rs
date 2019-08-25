mod clone_component;
pub use clone_component::CloneComponentFactory;
pub use clone_component::CloneComponentPrototype;
//pub use clone_component::CloneComponentPrototypeSerializer;

pub mod inspect;

pub mod persist;

mod prototype;
pub use prototype::PersistentComponentPrototype;
pub use prototype::PersistentEntityPrototype;
