use legion::prelude::*;

use minimum::resources::TimeResource;
use crate::resources::PhysicsResource;

use minimum::components::PositionComponent;
use crate::components::RigidBodyComponent;

pub fn update_physics() -> Box<dyn Schedulable> {
    // Do a physics simulation timestep
    SystemBuilder::new("update physics")
        .write_resource::<PhysicsResource>()
        .read_resource::<TimeResource>()
        .build(|_, _, (physics, time), _| {
            if time.is_simulation_paused() {
                physics.maintain()
            } else {
                physics.step();
            }
        })
}

pub fn read_from_physics() -> Box<dyn Schedulable> {
    SystemBuilder::new("read physics data")
        .read_resource::<PhysicsResource>()
        .with_query(<(Write<PositionComponent>, Read<RigidBodyComponent>)>::query())
        .build(|_, mut world, physics, query| {
            for (mut pos, body) in query.iter_mut(world) {
                if let Some(rigid_body) = physics.bodies.rigid_body(body.handle) {
                    let position = rigid_body.position().translation.vector;
                    pos.position.set_x(position.x);
                    pos.position.set_y(position.y);
                }
            }
        })
}
