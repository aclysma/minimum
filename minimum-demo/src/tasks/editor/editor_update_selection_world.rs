use minimum::resource::{DataRequirement, Write};

#[cfg(feature = "editor")]
use framework::resources::editor::EditorCollisionWorld;

use minimum::{Task, TaskContext};
use named_type::NamedType;

#[derive(NamedType)]
pub struct EditorUpdateSelectionWorld;
impl Task for EditorUpdateSelectionWorld {
    type RequiredResources = (Write<EditorCollisionWorld>);
    const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_SYSTEM as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut physics = data;
        physics.update();
    }
}
