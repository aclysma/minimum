mod registry;
pub use registry::SelectRegistry;
use crate::prototype::FrameworkEntityPrototypeInner;

/// Used for serialization of component prototypes
pub trait SelectableComponentPrototype<T>: Send + Sync {
    fn create_selection_shape(
        framework_entity: &FrameworkEntityPrototypeInner,
        data: &T,
    ) -> (
        ncollide2d::math::Isometry<f32>,
        ncollide2d::shape::ShapeHandle<f32>,
    );
}
