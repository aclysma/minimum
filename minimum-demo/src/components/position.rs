use crate::inspect::common_types::*;
use minimum::component::DefaultComponentReflector;
use minimum::component::VecComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
pub struct PositionComponent {
    #[inspect(proxy_type = "ImGlmVec2", on_set = "inspect_position_updated")]
    position: glm::Vec2,

    #[inspect(skip)]
    requires_sync_to_physics: bool,
}

impl PositionComponent {
    pub fn new(position: glm::Vec2) -> Self {
        PositionComponent {
            position,
            requires_sync_to_physics: false,
        }
    }

    pub fn position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.position
    }

    pub fn requires_sync_to_physics(&self) -> bool {
        self.requires_sync_to_physics
    }

    pub fn clear_requires_sync_to_physics(&mut self) {
        self.requires_sync_to_physics = false
    }

    //TODO: Replace with a better solution that doesn't require O(n) iteration
    // - Allow register callback on inspector?
    // - Some sort of flagging system?
    // - Attach an extra component?
    pub fn inspect_position_updated(&mut self) {
        self.requires_sync_to_physics = true;
    }
}

impl minimum::Component for PositionComponent {
    type Storage = VecComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}

use minimum::component::SlabComponentStorage;

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
    type Reflector = DefaultComponentReflector<Self>;
}
