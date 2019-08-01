use minimum::systems::{DataRequirement, Read};
use minimum::{Task, TaskContext};

use crate::resources::TimeState;

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::ComponentStorage;

#[derive(typename::TypeName)]
pub struct UpdatePositionWithVelocity;
impl Task for UpdatePositionWithVelocity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<TimeState>,
        WriteComponent<components::PositionComponent>,
        ReadComponent<components::VelocityComponent>,
        ReadComponent<components::PhysicsBodyComponent>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_PLAYING as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            time_state,
            mut position_components,
            velocity_components,
            physics_body_components,
        ) = data;

        let dt = time_state.playing().previous_frame_dt;

        for (entity, vel) in velocity_components.iter(&entity_set) {
            if physics_body_components.exists(&entity) {
                // Skip any entities that have a physics body as movement is being controlled by
                // nphysics
                continue;
            }

            if let Some(pos) = position_components.get_mut(&entity) {
                *pos.position_mut() += vel.velocity() * dt;
            }
        }
    }
}
