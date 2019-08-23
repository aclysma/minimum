use crate::inspect::common_types::*;
use minimum::component::DefaultComponentReflector;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
pub struct DebugDrawCircleComponent {
    #[inspect_slider(min_value = 0.0, max_value = 100.0)]
    radius: f32,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
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

impl minimum::Component for DebugDrawCircleComponent {
    type Storage = SlabComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
