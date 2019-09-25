use crate::resources;
use base::task::WriteAllTaskImpl;

use base::task::TaskConfig;
use base::ResourceMap;
use base::TaskContextFlags;

pub struct FrameworkUpdateActionQueue;
pub type FrameworkUpdateActionQueueTask = base::WriteAllTask<FrameworkUpdateActionQueue>;
impl WriteAllTaskImpl for FrameworkUpdateActionQueue {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<base::task::PhaseEndFrame>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        let mut framework_action_queue =
            resource_map.fetch_mut::<resources::FrameworkActionQueue>();
        framework_action_queue.process_queue(resource_map);
    }
}
