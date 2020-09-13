use legion::*;

use minimum::resources::TimeResource;
use crate::resources::PhysicsResource;

use minimum::components::TransformComponent;
use crate::components::RigidBodyComponent;
use crate::math_conversions::{vec2_glm_to_glam};

pub fn update_physics(schedule: &mut legion::systems::Builder) {
    // Do a physics simulation timestep
    schedule.add_system(
        SystemBuilder::new("update physics")
            .write_resource::<PhysicsResource>()
            .read_resource::<TimeResource>()
            .build(|_, _, (physics, time), _| {
                if time.is_simulation_paused() {
                    physics.maintain()
                } else {
                    physics.step();
                }
            }),
    );
}

pub fn read_from_physics(schedule: &mut legion::systems::Builder) {
    schedule.add_system(
        SystemBuilder::new("read physics data")
            .read_resource::<PhysicsResource>()
            .with_query(<(Write<TransformComponent>, Read<RigidBodyComponent>)>::query())
            .build(|_, world, physics, query| {
                for (transform, body) in query.iter_mut(world) {
                    if let Some(rigid_body) = physics.bodies.rigid_body(body.handle) {
                        let position = rigid_body.position().translation.vector;
                        //TODO: Conversion from 2D to 3D - ideally we'd use 3D physics with a constraint to force 2D
                        let v3 = vec2_glm_to_glam(position).extend(transform.position().z());
                        transform.set_position(v3);
                    }
                }
            }),
    );
}
