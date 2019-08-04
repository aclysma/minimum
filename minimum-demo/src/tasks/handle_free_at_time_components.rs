use minimum::resource::{DataRequirement, Read};
use minimum::{Task, TaskContext};

use crate::resources::TimeState;

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};

#[derive(typename::TypeName)]
pub struct HandleFreeAtTimeComponents;
impl Task for HandleFreeAtTimeComponents {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        WriteComponent<minimum::PendingDeleteComponent>,
        Read<TimeState>,
        ReadComponent<components::FreeAtTimeComponent>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_PLAYING as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
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
