#[cfg(feature = "editor")]
pub mod editor;

mod debug_draw_circle;
pub use debug_draw_circle::DebugDrawCircleComponent;

mod debug_draw_rect;
pub use debug_draw_rect::DebugDrawRectComponent;

//pub use editor::EditorSelectedComponent;
//pub use editor::EditorModifiedComponent;
//pub use editor::EditorShapeComponent;
//pub use editor::EditorShapeComponentFactory;
//pub use editor::EditorShapeComponentFreeHandler;
//pub use editor::EditorShapeComponentPrototype;

mod free_at_time;
pub use free_at_time::FreeAtTimeComponent;

mod persistent_entity;
pub use persistent_entity::PersistentEntityComponent;

pub mod transform;
pub use transform::TransformComponentPrototype;
pub use transform::TransformComponent;

mod velocity;
pub use velocity::VelocityComponent;