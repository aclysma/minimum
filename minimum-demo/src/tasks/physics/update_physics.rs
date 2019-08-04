use minimum::resource::{DataRequirement, Read, Write};

use crate::resources::{PhysicsManager, TimeState};

use minimum::{Task, TaskContext};

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
