use crate::base::task::WriteAllTaskImpl;
use crate::base::ResourceMap;
use crate::base::TaskConfig;
use crate::base::TaskContextFlags;

use crate::framework::resources::editor::EditorActionQueue;

pub struct EditorUpdateActionQueue;
pub type EditorUpdateActionQueueTask = crate::base::WriteAllTask<EditorUpdateActionQueue>;
impl WriteAllTaskImpl for EditorUpdateActionQueue {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseEndFrame>();
        config.this_provides_data_to::<crate::framework::tasks::FrameworkUpdateActionQueueTask>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        let mut editor_action_queue = resource_map.fetch_mut::<EditorActionQueue>();
        editor_action_queue.process_queue(resource_map);
    }
}
