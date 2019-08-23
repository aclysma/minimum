use crate::inspect::common_types::*;
use minimum::component::DefaultComponentReflector;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
pub struct DebugDrawRectComponent {
    #[inspect(proxy_type = "ImGlmVec2")]
    size: glm::Vec2,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
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
    type Reflector = DefaultComponentReflector<Self>;
}
