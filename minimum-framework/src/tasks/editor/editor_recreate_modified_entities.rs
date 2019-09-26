use crate::base::task::WriteAllTaskImpl;

use crate::base::task::TaskConfig;
use crate::base::ResourceMap;
use crate::base::TaskContextFlags;

use crate::components::editor::EditorSelectedComponent;

pub struct EditorRecreateModifiedEntities;
pub type EditorRecreateModifiedEntitiesTask = crate::base::WriteAllTask<EditorRecreateModifiedEntities>;
impl WriteAllTaskImpl for EditorRecreateModifiedEntities {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseEndFrame>();
        config.this_uses_data_from::<crate::tasks::FrameworkUpdateActionQueueTask>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        use crate::base::component::Component;
        use crate::base::component::ComponentStorage;

        let mut entity_set = resource_map.fetch_mut::<crate::base::EntitySet>();

        // Find all the modified persistent entities. Return a tuple of (prototypes, is_selected), and mark them for deletion
        // (the scoping here is intentional, we want to avoid having any active fetch when we call flush_free)
        let prototypes = {
            let persistent_entity_components = resource_map.fetch::<<crate::components::PersistentEntityComponent as Component>::Storage>();
            let editor_modified_components = resource_map.fetch::<<crate::components::editor::EditorModifiedComponent as Component>::Storage>();
            let editor_selected_components = resource_map.fetch::<<crate::components::editor::EditorSelectedComponent as Component>::Storage>();
            let mut pending_delete_components =
                resource_map.fetch_mut::<<crate::base::PendingDeleteComponent as Component>::Storage>();

            let mut prototypes = vec![];

            for (entity_handle, _editor_modified_component) in
                editor_modified_components.iter(&entity_set)
            {
                if let Some(persistent_entity_component) =
                    persistent_entity_components.get(&entity_handle)
                {
                    let prototype = persistent_entity_component.entity_prototype().clone();
                    let selected = editor_selected_components.exists(&entity_handle);

                    prototypes.push((prototype, selected));
                    entity_set.enqueue_free(&entity_handle, &mut *pending_delete_components);
                }
            }

            prototypes
        };

        if prototypes.is_empty() {
            return;
        }

        // Delete marked entities
        entity_set.flush_free(resource_map);

        //TODO: Validation? Ideally we can determine if a set of entities is improperly set up and
        // try to recover or roll back. At least, we can flag an error.

        // Recreate the entities (the scoping here is intentional, we want to avoid having any active fetch when we call flush_creates)
        {
            let mut editor_selected_components = resource_map.fetch_mut::<<crate::components::editor::EditorSelectedComponent as Component>::Storage>();
            for (prototype, is_selected) in prototypes {
                use crate::base::EntityPrototype;
                let entity = entity_set.allocate_get();
                prototype.create(resource_map, &entity);

                // If the entity was selected before it was deleted, re-select it
                if is_selected {
                    editor_selected_components
                        .allocate(&entity.handle(), EditorSelectedComponent::new())
                        .unwrap();
                }
            }
        }

        entity_set.flush_creates(resource_map);
    }
}
