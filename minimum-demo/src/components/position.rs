use minimum::component::VecComponentStorage;
use minimum::component::DefaultComponentReflector;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType)]
pub struct PositionComponent {
    position: glm::Vec2,
}

impl PositionComponent {
    pub fn new(position: glm::Vec2) -> Self {
        PositionComponent { position }
    }

    pub fn position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.position
    }
}

impl minimum::Component for PositionComponent {
    type Storage = VecComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
