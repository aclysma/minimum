use framework::inspect::common_types::*;
use imgui_inspect_derive::Inspect;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;
use serde::{Deserialize, Serialize};
use framework::select::SelectableComponentPrototype;

#[derive(Debug, Clone, NamedType, Inspect, Serialize, Deserialize)]
pub struct DebugDrawCircleComponent {
    #[inspect_slider(min_value = 0.1, max_value = 100.0)]
    radius: f32,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
}

impl Default for DebugDrawCircleComponent {
    fn default() -> Self {
        DebugDrawCircleComponent {
            radius: 10.0,
            color: glm::vec4(1.0, 1.0, 1.0, 1.0)
        }
    }
}

impl DebugDrawCircleComponent {
    pub fn new(radius: f32, color: glm::Vec4) -> Self {
        DebugDrawCircleComponent { radius, color }
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn color(&self) -> glm::Vec4 {
        self.color
    }
}

impl SelectableComponentPrototype<Self> for DebugDrawCircleComponent {
    fn create_selection_shape(data: &Self) -> (ncollide2d::math::Isometry<f32>, ncollide2d::shape::ShapeHandle<f32>) {
        use ncollide2d::shape::{Ball, ShapeHandle};
        (ncollide2d::math::Isometry::<f32>::new(glm::vec2(0.0, 0.0), 0.0), ShapeHandle::new(Ball::new(data.radius.max(std::f32::MIN_POSITIVE))))
    }
}

impl minimum::Component for DebugDrawCircleComponent {
    type Storage = SlabComponentStorage<Self>;
}
