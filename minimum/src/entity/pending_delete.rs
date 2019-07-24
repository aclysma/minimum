
use minimum::component::SlabComponentStorage;

#[derive(Debug)]
pub struct PendingDeleteComponent {
    velocity: glm::Vec2
}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        VelocityComponent {
            velocity
        }
    }

    pub fn velocity(&self) -> glm::Vec2 {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut glm::Vec2 { &mut self.velocity }
}

impl minimum::Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<VelocityComponent>;
}
