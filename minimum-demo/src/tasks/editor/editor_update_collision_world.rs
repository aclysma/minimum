use minimum::systems::{DataRequirement, Write};

use crate::resources::EditorCollisionWorld;

use minimum::{Task, TaskContext};

#[derive(typename::TypeName)]
pub struct EditorUpdateCollisionWorld;
impl Task for EditorUpdateCollisionWorld {
    type RequiredResources = (Write<EditorCollisionWorld>);
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_SYSTEM as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut physics = data;
        physics.update();
    }
}
