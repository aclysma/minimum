pub mod common_types;
mod registry;

pub use registry::InspectRegistry;

#[derive(PartialEq, Debug)]
pub enum InspectorTab {
    Persistent = 0,
    Runtime = 1
}

use minimum::ResourceMap;
use minimum::Component;
use minimum::ComponentStorage;
use crate::resources;
use crate::components;

use components::PersistentEntityComponent;
use crate::components::editor::EditorModifiedComponent;

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
            resource_map.fetch_mut::<<components::editor::EditorSelectedComponent as Component>::Storage>();
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

                use imgui::ImGuiSelectableFlags;
                let mut component_types : Vec<_> = persist_registry.iter_names().collect();
                component_types.sort_by(|(_, str1), (_, str2)| str1.cmp(str2));

                for (type_id, component_name) in component_types {
                    if editor_ui_state.add_component_search_text.is_empty() ||
                        component_name.contains(editor_ui_state.add_component_search_text.to_str())
                    {
                        if ui.selectable(&im_str!("{}", component_name), false, ImGuiSelectableFlags::empty(), [0.0, 0.0]) {
                            selected_type_id = Some(type_id.clone());
                        }
                    }
                }

                if let Some(type_id) = selected_type_id {
                    // Get storages for the components we will modify
                    let mut prototype_components = resource_map.fetch_mut::<<PersistentEntityComponent as Component>::Storage>();
                    let mut modified_components = resource_map.fetch_mut::<<components::editor::EditorModifiedComponent as Component>::Storage>();

                    for selected_entity_handle in &selected_entity_handles {
                        //TODO: Check that one doesn't exist already or switch to using hashmap
                        if let Some(pec) = prototype_components.get_mut(selected_entity_handle) {

                            // Create a new instance of the component
                            let default_component = persist_registry.create_default(type_id);

                            // Get the entity prototype for the component so that we can modify it
                            let entity_prototype = pec.entity_prototype_mut();
                            let mut entity_prototype_guard = entity_prototype.get_mut();

                            // Add the component to the entity prototype
                            entity_prototype_guard.component_prototypes_mut().push(default_component);

                            // Mark the entity as needed to be regenerated
                            entity_set.get_entity_ref(selected_entity_handle).unwrap().add_component(&mut *modified_components, EditorModifiedComponent::new());
                        }
                    }
                }
            });

            inspect_registry.render_mut(resource_map, selected_entity_handles.as_slice(), ui, &mut editor_ui_state.set_inspector_tab);
        }
    )
}