use framework::inspect::common_types::*;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType)]
pub struct PositionChangedComponent {
    position: glm::Vec2,
}

impl PositionChangedComponent {
    pub fn new(position: glm::Vec2) -> Self {
        PositionChangedComponent { position }
    }
}

impl minimum::Component for PositionChangedComponent {
    type Storage = SlabComponentStorage<Self>;
}
