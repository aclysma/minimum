pub mod common_types;
mod registry;

pub use registry::InspectRegistry;

#[derive(PartialEq, Debug)]
pub enum InspectorTab {
    Persistent = 0,
    Runtime = 1,
}

use crate::components;
use crate::resources;
use minimum::Component;
use minimum::ComponentStorage;
use minimum::ResourceMap;

use crate::components::editor::EditorModifiedComponent;
use components::PersistentEntityComponent;

pub fn draw_inspector(resource_map: &ResourceMap, ui: &imgui::Ui) {
    let play_mode = resource_map.fetch::<resources::TimeState>().play_mode;
    let mut editor_ui_state = resource_map.fetch_mut::<resources::editor::EditorUiState>();
    let window_options = editor_ui_state.window_options(play_mode);
    if !window_options.show_inspector {
        return;
    }

    let entity_set = resource_map.fetch::<minimum::EntitySet>();
    let selected_entity_handles = {
        let selected_components =
            resource_map
                .fetch_mut::<<components::editor::EditorSelectedComponent as Component>::Storage>();
        let mut selected = vec![];
        for (entity_handle, _) in selected_components.iter(&entity_set) {
            selected.push(entity_handle);
        }
        selected
    };

    //TODO: Need a way to delete components

    let inspect_registry = resource_map.fetch::<crate::inspect::InspectRegistry>();
    let persist_registry = resource_map.fetch::<crate::persist::PersistRegistry>();
    //let mut imgui_manager = resource_map.fetch_mut::<resources::ImguiManager>();

    use imgui::im_str;

    //ui.set
    imgui::Window::new(im_str!("Inspector"))
        .position([0.0, 350.0], imgui::Condition::Once)
        .size([350.0, 500.0], imgui::Condition::Once)
        .build(ui, || {
            if ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]) {
                //ui.open_popup(im_str!("Add Component"));
                ui.open_popup(im_str!("Add Component"));
            }

            ui.popup(im_str!("Add Component"), || {
                ui.input_text(
                    im_str!("Filter"),
                    &mut editor_ui_state.add_component_search_text)
                    .resize_buffer(true)
                    .build();

                let mut selected_type_id = None;

                let mut component_types : Vec<_> = persist_registry.iter_metadata().collect();
                component_types.sort_by(|t1, t2| t1.name().cmp(t2.name()));

                // An array of bools the same length as the components.
                let mut can_add_to_some_entity = Vec::with_capacity(component_types.len());
                can_add_to_some_entity.resize(component_types.len(), false);

                // Get storages for the components we will modify
                let mut prototype_components = resource_map.fetch_mut::<<PersistentEntityComponent as Component>::Storage>();
                let mut modified_components = resource_map.fetch_mut::<<components::editor::EditorModifiedComponent as Component>::Storage>();

                // Iterate all entities and see if there is any entity that can accept each component type.
                // This is used to disable entities that can't be added
                for selected_entity_handle in &selected_entity_handles {
                    if let Some(pec) = prototype_components.get(selected_entity_handle) {
                        // Get the entity prototype for the component so that we can modify it
                        let entity_prototype = pec.entity_prototype();
                        let entity_prototype_guard = entity_prototype.lock();

                        // Add the component to the entity prototype
                        for i in 0..component_types.len() {
                            // If we already know an entity is able to accept the component type, we can skip
                            // checking any other entities
                            if !can_add_to_some_entity[i] {
                                let metadata = component_types[i];
                                let component_id = metadata.component_type_id();
                                
                                if entity_prototype_guard.find_component_prototype_by_component_type_id(*component_id).is_none() {
                                    can_add_to_some_entity[i] = true;
                                }
                            }
                        }
                    }
                }

                // Draw the menu options
                //TODO: Consider drawing by hierarchy of component type.. i.e. PhysicsComponent -> PhysicsComponentBoxPrototype
                for i in 0..component_types.len() {
                    let metadata = &component_types[i];
                    if editor_ui_state.add_component_search_text.is_empty() ||
                        metadata.name().contains(editor_ui_state.add_component_search_text.to_str())
                    {
                        if imgui::Selectable::new(&im_str!("{}", metadata.name())).disabled(!can_add_to_some_entity[i]).build(ui) {
                            selected_type_id = Some(metadata.type_id().clone());
                        }
                    }
                }

                // Check if anythign was clicked. If it was, try adding the component to all entities
                if let Some(type_id) = selected_type_id {
                    for selected_entity_handle in &selected_entity_handles {
                        if let Some(pec) = prototype_components.get_mut(selected_entity_handle) {

                            // Get the entity prototype for the component so that we can modify it
                            let entity_prototype = pec.entity_prototype_mut();
                            let mut entity_prototype_guard = entity_prototype.lock();

                            // Skip this entity if the component already exists
                            if entity_prototype_guard.find_component_prototype_by_component_type_id(type_id).is_some() {
                                //TODO: Log or otherwise message to user that some objects already have the component
                                continue;
                            }

                            // Create a new instance of the component
                            let default_component = persist_registry.create_default(type_id);

                            // Add the component to the entity prototype
                            entity_prototype_guard.component_prototypes_mut().push(default_component);

                            // Mark the entity as needed to be regenerated
                            entity_set
                                .get_entity_ref(selected_entity_handle)
                                .unwrap()
                                .add_component(&mut *modified_components, EditorModifiedComponent::new())
                                .unwrap();
                        }
                    }
                }
            });

            inspect_registry.render_mut(resource_map, selected_entity_handles.as_slice(), ui, &mut editor_ui_state.set_inspector_tab);
        }
    )
}
