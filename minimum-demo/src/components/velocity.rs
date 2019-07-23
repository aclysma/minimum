use minimum::component::SlabComponentStorage;

#[derive(Debug)]
pub struct VelocityComponent {
    velocity: glm::Vec2
}

impl VelocityComponent {
    pub fn new(velocity: glm::Vec2) -> Self {
        VelocityComponent {
            velocity
        }
    }

    pub fn velocity(&self) -> glm::Vec2 {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut glm::Vec2 { &mut self.velocity }
}

impl minimum::Component for VelocityComponent {
    type Storage = SlabComponentStorage<VelocityComponent>;
}
