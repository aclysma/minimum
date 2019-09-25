#[cfg(feature = "editor")]
use crate::inspect::common_types::*;

#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;

#[cfg(feature = "editor")]
use crate::select::SelectableComponentPrototype;
use minimum::component::SlabComponentStorage;
use crate::FrameworkEntityPrototypeInner;
use crate::components::transform;
use crate::components::TransformComponentPrototype;

type RectSize = transform::Scale;
type ImRectSize = transform::ImScale;


#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct DebugDrawRectComponent {
    #[inspect(proxy_type = "ImRectSize")]
    size: RectSize,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
}

impl Default for DebugDrawRectComponent {
    fn default() -> Self {
        DebugDrawRectComponent {
            size: transform::default_scale(),
            color: glm::vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl DebugDrawRectComponent {
    pub fn new(size: RectSize, color: glm::Vec4) -> Self {
        DebugDrawRectComponent { size, color }
    }

    pub fn size(&self) -> RectSize {
        self.size
    }

    pub fn color(&self) -> glm::Vec4 {
        self.color
    }
}

#[cfg(feature = "editor")]
impl SelectableComponentPrototype<Self> for DebugDrawRectComponent {
    fn create_selection_shape(
        framework_entity: &FrameworkEntityPrototypeInner,
        data: &Self,
    ) -> (
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    ) {

        let mut scale = transform::default_scale();
        if let Some(transform) = framework_entity.find_component_prototype::<TransformComponentPrototype>() {
            scale = transform.data().scale();
        }

        use ncollide::shape::{Cuboid, ShapeHandle};
        let extents = scale.component_mul(&data.size);
        (
            ncollide::math::Isometry::<f32>::new(glm::zero(), glm::zero()),
            ShapeHandle::new(Cuboid::new(extents / 2.0)),
        )
    }
}

impl minimum::Component for DebugDrawRectComponent {
    type Storage = SlabComponentStorage<Self>;
}
