#[cfg(feature = "editor")]
use framework::inspect::common_types::*;

#[cfg(feature = "editor")]
use framework::select::SelectableComponentPrototype;
#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use minimum::component::SlabComponentStorage;

#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
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
            color: glm::vec4(1.0, 1.0, 1.0, 1.0),
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

#[cfg(feature = "editor")]
impl SelectableComponentPrototype<Self> for DebugDrawCircleComponent {
    fn create_selection_shape(
        data: &Self,
    ) -> (
        ncollide2d::math::Isometry<f32>,
        ncollide2d::shape::ShapeHandle<f32>,
    ) {
        let scale = glm::vec2(1.0, 1.0);
        let mut radius = data.radius * f32::max(scale.x, scale.y);
        if radius < std::f32::MIN_POSITIVE {
            warn!("Tried to create a circle with <=0 radius");
            radius = std::f32::MIN_POSITIVE;
        }

        use ncollide2d::shape::{Ball, ShapeHandle};
        (
            ncollide2d::math::Isometry::<f32>::new(glm::vec2(0.0, 0.0), 0.0),
            ShapeHandle::new(Ball::new(radius)),
        )
    }
}

impl minimum::Component for DebugDrawCircleComponent {
    type Storage = SlabComponentStorage<Self>;
}
