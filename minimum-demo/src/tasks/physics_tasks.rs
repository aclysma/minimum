use minimum::systems::{DataRequirement, Read, Write};

use crate::resources::{PhysicsManager, TimeState};

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::{ComponentStorage, EntitySet, Task, TaskContext};

#[derive(typename::TypeName)]
pub struct UpdatePhysics;
impl Task for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_PLAYING as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}

#[derive(typename::TypeName)]
pub struct UpdatePositionFromPhysics;
impl Task for UpdatePositionFromPhysics {
    type RequiredResources = (
        Read<EntitySet>,
        Read<PhysicsManager>,
        ReadComponent<components::PhysicsBodyComponent>,
        WriteComponent<components::PositionComponent>,
        WriteComponent<components::VelocityComponent>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_PLAYING as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            physics_manager,
            physics_body_components,
            mut pos_components,
            mut vel_components,
        ) = data;

        for (entity, body_component) in physics_body_components.iter(&entity_set) {
            let body: &nphysics2d::object::RigidBody<f32> = physics_manager
                .world()
                .rigid_body(body_component.body_handle())
                .unwrap();

            if let Some(pos_component) = pos_components.get_mut(&entity) {
                *pos_component.position_mut() = body.position().translation.vector;
            }

            if let Some(vel_component) = vel_components.get_mut(&entity) {
                *vel_component.velocity_mut() = body.velocity().linear;
            }
        }
    }
}
