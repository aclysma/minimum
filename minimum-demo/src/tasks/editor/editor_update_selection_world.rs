use minimum::resource::{DataRequirement, Write};

#[cfg(feature = "editor")]
use framework::resources::editor::EditorCollisionWorld;

use minimum::{ResourceTaskImpl, TaskConfig};
use named_type::NamedType;

#[derive(NamedType)]
pub struct EditorUpdateSelectionWorld;
pub type EditorUpdateSelectionWorldTask = minimum::ResourceTask<EditorUpdateSelectionWorld>;
impl ResourceTaskImpl for EditorUpdateSelectionWorld {
    type RequiredResources = (Write<EditorCollisionWorld>);
    //const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_SYSTEM as usize;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut physics = data;
        physics.update();
    }
}
