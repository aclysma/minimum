use minimum::systems::{DataRequirement, Write};
use minimum::{Task, TaskContext};

use crate::resources::DebugDraw;

#[derive(typename::TypeName)]
pub struct ClearDebugDraw;
impl Task for ClearDebugDraw {
    type RequiredResources = (Write<DebugDraw>);
    const REQUIRED_FLAGS: usize = crate::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut debug_draw = data;

        debug_draw.clear();
    }
}
