use crate::base::resource::{DataRequirement, Write};

#[cfg(feature = "editor")]
use crate::framework::resources::editor::EditorCollisionWorld;

use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

pub struct EditorUpdateSelectionWorld;
pub type EditorUpdateSelectionWorldTask = crate::base::ResourceTask<EditorUpdateSelectionWorld>;
impl ResourceTaskImpl for EditorUpdateSelectionWorld {
    type RequiredResources = (Write<EditorCollisionWorld>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePreRender>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut physics = data;
        physics.update();
    }
}
