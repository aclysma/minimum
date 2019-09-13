use minimum::resource::{DataRequirement, Write};

#[cfg(feature = "editor")]
use framework::resources::editor::EditorCollisionWorld;

use minimum::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

pub struct EditorUpdateSelectionWorld;
pub type EditorUpdateSelectionWorldTask = minimum::ResourceTask<EditorUpdateSelectionWorld>;
impl ResourceTaskImpl for EditorUpdateSelectionWorld {
    type RequiredResources = (Write<EditorCollisionWorld>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.run_only_if(framework::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut physics = data;
        physics.update();
    }
}
