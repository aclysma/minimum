use crate::base::resource::{DataRequirement, Read, Write};

use crate::resources::PhysicsManager;
use crate::framework::resources::TimeState;

use crate::base::{ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags};

pub struct UpdatePhysics;
pub type UpdatePhysicsTask = ResourceTask<UpdatePhysics>;
impl ResourceTaskImpl for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePhysics>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}
