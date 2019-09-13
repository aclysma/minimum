use minimum::resource::{DataRequirement, Read, Write};

use crate::resources::PhysicsManager;
use framework::resources::TimeState;

use minimum::{ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags};

pub struct UpdatePhysics;
pub type UpdatePhysicsTask = ResourceTask<UpdatePhysics>;
impl ResourceTaskImpl for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePhysics>();
        config.run_only_if(framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}
