
use minimum::task::WriteAllTaskImpl;
use minimum::TaskConfig;
use minimum::ResourceMap;

use framework::resources::editor::EditorActionQueue;

pub struct EditorUpdateActionQueue;
pub type EditorUpdateActionQueueTask = minimum::WriteAllTask<EditorUpdateActionQueue>;
impl WriteAllTaskImpl for EditorUpdateActionQueue {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseEndFrame>();
        config.this_provides_data_to::<framework::tasks::FrameworkUpdateActionQueueTask>();
    }

    fn run(resource_map: &mut ResourceMap) {
        let mut editor_action_queue = resource_map.fetch_mut::<EditorActionQueue>();
        editor_action_queue.process_queue(resource_map);
    }
}
