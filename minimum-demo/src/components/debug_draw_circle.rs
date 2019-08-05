use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType)]
pub struct DebugDrawCircleComponent {
    radius: f32,
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
