use minimum::resource::{DataRequirement, Write};
use minimum::{ResourceTask, ResourceTaskImpl, TaskConfig};

use crate::resources::DebugDraw;

pub struct ClearDebugDraw;
pub type ClearDebugDrawTask = ResourceTask<ClearDebugDraw>;
impl ResourceTaskImpl for ClearDebugDraw {
    type RequiredResources = (Write<DebugDraw>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseFrameBegin>();
    }
    //const REQUIRED_FLAGS: usize = framework::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        //&self,
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut debug_draw = data;

        debug_draw.clear();
    }
}

