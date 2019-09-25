use base::task::ReadAllTaskImpl;
use base::EntitySet;
use base::ResourceMap;
use base::TaskConfig;
use base::TaskContextFlags;

pub struct UpdateEntitySet;
pub type UpdateEntitySetTask = base::ReadAllTask<UpdateEntitySet>;
impl ReadAllTaskImpl for UpdateEntitySet {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<base::task::PhaseEndFrame>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &ResourceMap) {
        let mut entity_set = resource_map.fetch_mut::<EntitySet>();
        entity_set.update(resource_map);
    }
}
