use base::resource::{DataRequirement, Write};
use base::{ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::resources::DebugDraw;

pub struct ClearDebugDraw;
pub type ClearDebugDrawTask = ResourceTask<ClearDebugDraw>;
impl ResourceTaskImpl for ClearDebugDraw {
    type RequiredResources = (Write<DebugDraw>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<base::task::PhaseFrameBegin>();
        config.run_only_if(crate::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut debug_draw = data;

        debug_draw.clear();
    }
}
