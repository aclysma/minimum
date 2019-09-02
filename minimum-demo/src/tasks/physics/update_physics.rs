use minimum::resource::{DataRequirement, Read, Write};

use framework::resources::TimeState;
use crate::resources::PhysicsManager;

use minimum::{Task, TaskContext};
use named_type::NamedType;

#[derive(NamedType)]
pub struct UpdatePhysics;
impl Task for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);
    const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_PLAYING as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}
