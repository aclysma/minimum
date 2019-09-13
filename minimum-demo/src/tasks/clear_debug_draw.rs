use minimum::resource::{DataRequirement, Write};
use minimum::{ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::resources::DebugDraw;

pub struct ClearDebugDraw;
pub type ClearDebugDrawTask = ResourceTask<ClearDebugDraw>;
impl ResourceTaskImpl for ClearDebugDraw {
    type RequiredResources = (Write<DebugDraw>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseFrameBegin>();
        config.run_only_if(framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut debug_draw = data;

        debug_draw.clear();
    }
}

