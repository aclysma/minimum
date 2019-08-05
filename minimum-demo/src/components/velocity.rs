use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType)]
pub struct VelocityComponent {
    velocity: glm::Vec2,
}

impl VelocityComponent {
    pub fn new(velocity: glm::Vec2) -> Self {
        VelocityComponent { velocity }
    }

    pub fn velocity(&self) -> glm::Vec2 {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.velocity
    }
}

impl minimum::Component for VelocityComponent {
    type Storage = SlabComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
