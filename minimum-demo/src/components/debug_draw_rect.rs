use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;

#[derive(Debug, Clone, typename::TypeName)]
pub struct DebugDrawRectComponent {
    size: glm::Vec2,
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
