#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use base::component::SlabComponentStorage;
use crate::components::transform;

pub type Velocity = transform::Position;
#[cfg(feature = "editor")]
pub type ImVelocity = transform::ImPosition;

#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct VelocityComponent {
    #[inspect(proxy_type = "ImVelocity")]
    velocity: Velocity,

    #[inspect(skip)]
    requires_sync_to_physics: bool,
}

impl VelocityComponent {
    pub fn new(velocity: Velocity) -> Self {
        VelocityComponent {
            velocity,
            requires_sync_to_physics: false,
        }
    }

    pub fn velocity(&self) -> Velocity {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut Velocity {
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
            requires_sync_to_physics: false,
        }
    }
}

impl base::Component for VelocityComponent {
    type Storage = SlabComponentStorage<Self>;
}
