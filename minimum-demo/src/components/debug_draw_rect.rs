use crate::framework::inspect::common_types::*;
use imgui_inspect_derive::Inspect;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, NamedType, Inspect, Serialize, Deserialize)]
pub struct DebugDrawRectComponent {
    #[inspect(proxy_type = "ImGlmVec2")]
    size: glm::Vec2,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
}

impl Default for DebugDrawRectComponent {
    fn default() -> Self {
        DebugDrawRectComponent {
            size: glm::vec2(10.0, 10.0),
            color: glm::vec4(1.0, 1.0, 1.0, 1.0)
        }
    }
}

impl DebugDrawRectComponent {
    pub fn new(size: glm::Vec2, color: glm::Vec4) -> Self {
        DebugDrawRectComponent { size, color }
    }

    pub fn size(&self) -> glm::Vec2 {
        self.size
    }

    pub fn color(&self) -> glm::Vec4 {
        self.color
    }
}

impl minimum::Component for DebugDrawRectComponent {
    type Storage = SlabComponentStorage<Self>;
}
