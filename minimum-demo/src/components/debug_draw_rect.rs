use minimum::component::SlabComponentStorage;

#[derive(Debug, Clone)]
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
    type Storage = SlabComponentStorage<DebugDrawRectComponent>;
}
