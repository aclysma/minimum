use framework::inspect::common_types::*;
use minimum::component::SlabComponentStorage;

#[derive(Debug, Clone)]
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
