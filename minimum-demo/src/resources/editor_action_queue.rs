
use std::collections::VecDeque;
use minimum::{ResourceMap, EntitySet, Component, PendingDeleteComponent, EntityPrototype};
use crate::components::EditorSelectedComponent;
use crate::framework::{FrameworkEntityPrototype, FrameworkEntityPersistencePolicy};

pub struct EditorActionQueue {
    queue: VecDeque<fn(&ResourceMap)>
}

impl EditorActionQueue {
    pub fn new() -> Self {
        EditorActionQueue {
            queue: VecDeque::new()
        }
    }

    pub fn enqueue_add_new_entity(&mut self) {
        self.queue.push_back(move |resource_map| {
            let mut entity_set = resource_map.fetch_mut::<EntitySet>();

            // Create the entity and enqueue adding the components
            {
                let mut editor_selected_components = resource_map.fetch_mut::<<EditorSelectedComponent as Component>::Storage>();
                editor_selected_components.free_all();

                let pec = FrameworkEntityPrototype::new(
                    std::path::PathBuf::from("testpath"),
                    FrameworkEntityPersistencePolicy::Persistent,
                    vec![],
                );

                let entity_ref = entity_set.allocate_get();
                pec.create(resource_map, &entity_ref);
                entity_ref.add_component(&mut *editor_selected_components, EditorSelectedComponent::new());
            }

            entity_set.flush_creates(resource_map);
        });
    }

    pub fn enqueue_delete_selected_entities(&mut self) {
        self.queue.push_back(move |resource_map| {
            let mut entity_set = resource_map.fetch_mut::<EntitySet>();

            // Mark everything we wish to free
            {
                let editor_selected_components = resource_map.fetch::<<EditorSelectedComponent as Component>::Storage>();
                let mut pending_delete_components = resource_map.fetch_mut::<<PendingDeleteComponent as Component>::Storage>();

                for (entity_handle, _c) in editor_selected_components.iter(&entity_set)
                    {
                        entity_set
                            .enqueue_free(&entity_handle, &mut *pending_delete_components);
                    }
            }

            // Free it
            entity_set.flush_free(resource_map);
        });
    }

    pub fn process_queue(&mut self, resource_map: &ResourceMap) {
        for action in self.queue.drain(..) {
            (action)(resource_map);
        }
    }
}