
#[cfg(feature = "editor")]
pub mod editor;

//pub use editor::EditorSelectedComponent;
//pub use editor::EditorModifiedComponent;
//pub use editor::EditorShapeComponent;
//pub use editor::EditorShapeComponentFactory;
//pub use editor::EditorShapeComponentFreeHandler;
//pub use editor::EditorShapeComponentPrototype;

mod persistent_entity;
pub use persistent_entity::PersistentEntityComponent;