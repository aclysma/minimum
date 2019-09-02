
#[cfg(feature = "editor")]
use framework::inspect::common_types::*;

#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, NamedType, Serialize, Deserialize, Inspect)]
pub struct VelocityComponent {
    #[inspect(proxy_type = "ImGlmVec2")]
    velocity: glm::Vec2,

    #[inspect(skip)]
    requires_sync_to_physics: bool,
}

impl VelocityComponent {
    pub fn new(velocity: glm::Vec2) -> Self {
        VelocityComponent {
            velocity,
            requires_sync_to_physics: false,
        }
    }

    pub fn velocity(&self) -> glm::Vec2 {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.velocity
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

impl Default for VelocityComponent {
    fn default() -> Self {
        VelocityComponent {
            velocity: glm::zero(),
            requires_sync_to_physics: false
        }
    }
}

impl minimum::Component for VelocityComponent {
    type Storage = SlabComponentStorage<Self>;
}
