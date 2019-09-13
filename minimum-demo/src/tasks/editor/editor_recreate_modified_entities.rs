use minimum::task::WriteAllTaskImpl;

use minimum::task::TaskConfig;
use minimum::ResourceMap;
use minimum::TaskContextFlags;

use framework::components::editor::EditorSelectedComponent;

pub struct EditorRecreateModifiedEntities;
pub type EditorRecreateModifiedEntitiesTask = minimum::WriteAllTask<EditorRecreateModifiedEntities>;
impl WriteAllTaskImpl for EditorRecreateModifiedEntities {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseEndFrame>();
        config.this_uses_data_from::<framework::tasks::FrameworkUpdateActionQueueTask>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        use minimum::component::Component;
        use minimum::component::ComponentStorage;

        let mut entity_set = resource_map.fetch_mut::<minimum::EntitySet>();

        // Find all the modified persistent entities. Return a tuple of (prototypes, is_selected), and mark them for deletion
        // (the scoping here is intentional, we want to avoid having any active fetch when we call flush_free)
        let prototypes = {
            let persistent_entity_components = resource_map.fetch::<<framework::components::PersistentEntityComponent as Component>::Storage>();
            let editor_modified_components = resource_map.fetch::<<framework::components::editor::EditorModifiedComponent as Component>::Storage>();
            let editor_selected_components = resource_map.fetch::<<framework::components::editor::EditorSelectedComponent as Component>::Storage>();
            let mut pending_delete_components =
                resource_map.fetch_mut::<<minimum::PendingDeleteComponent as Component>::Storage>();

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

        // Recreate the entities (the scoping here is intentional, we want to avoid having any active fetch when we call flush_creates)
        {
            let mut editor_selected_components = resource_map.fetch_mut::<<framework::components::editor::EditorSelectedComponent as Component>::Storage>();
            for (prototype, is_selected) in prototypes {
                use minimum::EntityPrototype;
                let entity = entity_set.allocate_get();
                prototype.create(resource_map, &entity);

                // If the entity was selected before it was deleted, re-select it
                if is_selected {
                    editor_selected_components
                        .allocate(&entity.handle(), EditorSelectedComponent::new());
                }
            }
        }

        entity_set.flush_creates(resource_map);
    }
}
