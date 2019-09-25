use base::resource::{DataRequirement, Read};
use base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::resources::TimeState;

use base::component::{ReadComponent, WriteComponent};

pub struct HandleFreeAtTimeComponents;
pub type HandleFreeAtTimeComponentsTask = base::ResourceTask<HandleFreeAtTimeComponents>;
impl ResourceTaskImpl for HandleFreeAtTimeComponents {
    type RequiredResources = (
        Read<base::EntitySet>,
        WriteComponent<base::PendingDeleteComponent>,
        Read<TimeState>,
        ReadComponent<crate::components::FreeAtTimeComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<base::task::PhaseFrameBegin>();
        config.run_only_if(crate::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (entity_set, mut write_components, time_state, free_at_time_components) = data;

        //TODO-API: Find a better way to do this.. deferred delete is fine
        let mut entities_to_free = vec![];
        for (entity, free_at_time) in free_at_time_components.iter(&entity_set) {
            if free_at_time.should_free(&time_state) {
                entities_to_free.push(entity);
            }
        }

        for e in entities_to_free {
            entity_set.enqueue_free(&e, &mut *write_components);
        }
    }
}
