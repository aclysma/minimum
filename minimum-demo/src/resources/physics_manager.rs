use crate::resources;

const GRAVITY: f32 = 0.0;

//TODO: Are steps really necessary? Appropriate size?
const STEP_SIZE: f32 = 1.0 / 120.0;

pub struct PhysicsManager {
    world: nphysics2d::world::World<f32>,

    time_accumulator: f32,
}

impl PhysicsManager {
    pub fn new() -> Self {
        let mut world = nphysics2d::world::World::<f32>::new();
        let gravity = glm::Vec2::y() * GRAVITY;
        world.set_gravity(gravity);
        world.integration_parameters_mut().dt = STEP_SIZE;

        PhysicsManager {
            world,
            time_accumulator: 0.0,
        }
    }

    pub fn update(&mut self, time_state: &resources::TimeState) {
        self.time_accumulator += time_state.previous_frame_dt;
        while self.time_accumulator > STEP_SIZE {
            self.world.step();
            self.time_accumulator -= STEP_SIZE;
        }
    }

    pub fn world(&self) -> &nphysics2d::world::World<f32> {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut nphysics2d::world::World<f32> {
        &mut self.world
    }
}
