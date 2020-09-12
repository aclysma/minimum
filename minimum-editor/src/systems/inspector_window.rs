use legion::*;

use minimum_game::resources::{ImguiResource};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, PostCommitSelection,
    EditorInspectRegistryResource,
};

use imgui;
use imgui::im_str;

use std::collections::HashMap;

use prefab_format::{EntityUuid};

use minimum_kernel::resources::ComponentRegistryResource;
use minimum_kernel::resources::AssetResource;

pub fn editor_inspector_window(
    _world: &mut World,
    resources: &mut Resources,
) {
    {
        let selection_world = resources.get::<EditorSelectionResource>().unwrap();

        let imgui_manager = resources.get::<ImguiResource>().unwrap();

        let mut editor_ui_state = resources.get_mut::<EditorStateResource>().unwrap();
        let asset_resource = resources.get::<AssetResource>().unwrap();

        let opened_prefab = editor_ui_state.opened_prefab();
        if opened_prefab.is_none() {
            return;
        }

        let opened_prefab = opened_prefab.unwrap();

        // Create a lookup from prefab entity to the entity UUID
        use std::iter::FromIterator;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            let window_options = editor_ui_state.window_options();

            if window_options.show_inspector {
                imgui::Window::new(im_str!("Inspector"))
                    .position([0.0, 300.0], imgui::Condition::Once)
                    .size([350.0, 300.0], imgui::Condition::Once)
                    .build(ui, || {
                        let component_registry =
                            resources.get::<ComponentRegistryResource>().unwrap();
                        let tx = editor_ui_state.create_transaction_from_selected(
                            &*selection_world,
                            &*component_registry,
                        );

                        if let Some(mut tx) = tx {
                            // Draw a button to bring up the add component menu
                            if ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]) {
                                ui.open_popup(im_str!("Add Component"));
                            }

                            let mut component_type_to_add = None;

                            // Render the add component pop-up. It has a filtering text box and lists
                            // component types that can be clicked
                            ui.popup(im_str!("Add Component"), || {
                                // Draw the filter text box
                                ui.input_text(
                                    im_str!("Filter"),
                                    &mut editor_ui_state.add_component_search_text,
                                )
                                .resize_buffer(true)
                                .build();

                                // Lowercase the text to do a case-insensitive compare
                                let filter_string = editor_ui_state
                                    .add_component_search_text
                                    .to_str()
                                    .to_lowercase();

                                // Get a list of all component types that match the filter (or don't filter if the string is empty)
                                let mut component_types: Vec<_> = component_registry
                                    .components()
                                    .iter()
                                    .filter(|(_, t)| {
                                        filter_string.is_empty()
                                            || t.type_name().to_lowercase().contains(&filter_string)
                                    })
                                    .collect();

                                // Sort components alphabetically
                                component_types
                                    .sort_by(|(_, t1), (_, t2)| t1.type_name().cmp(t2.type_name()));

                                let mut can_add_to_some_entity =
                                    Vec::with_capacity(component_types.len());
                                can_add_to_some_entity.resize(component_types.len(), false);

                                //TODO: Determine what components can be added - Consider the type that this would become when spawned
                                for (index, (_, component_type)) in
                                    component_types.iter().enumerate()
                                {
                                    let mut all = Entity::query();
                                    for entity in all.iter(tx.world()) {
                                        if !tx
                                            .world()
                                            .entry_ref(*entity)
                                            .unwrap()
                                            .archetype()
                                            .layout()
                                            .has_component_by_id(component_type.component_type_id())
                                        {
                                            can_add_to_some_entity[index] = true;
                                            break;
                                        }
                                    }
                                }

                                // Draw all the menu items, if one of them is clicked store it in component_type_to_add
                                //TODO: Consider drawing by hierarchy of component type.. i.e. PhysicsComponent -> PhysicsComponentBoxPrototype
                                for (index, (_, v)) in component_types.iter().enumerate() {
                                    let disabled = !can_add_to_some_entity[index];
                                    if imgui::Selectable::new(&im_str!("{}", v.type_name()))
                                        .disabled(disabled)
                                        .build(ui)
                                    {
                                        component_type_to_add = Some(v.clone());
                                    }
                                }
                            });

                            let mut commit_required = false;

                            // Make a list of all entities, this is necessary because we can't take an &-borrow for uuid_to_entities at the
                            // same time as an &mut-borrow for world_mut()
                            let mut all = Entity::query();
                            let all_entities: Vec<Entity> =
                                all.iter(tx.world()).map(|x| *x).collect();

                            //
                            // If a component needs to be added, do that now
                            //
                            if let Some(component_type_to_add) = component_type_to_add {
                                //TODO: Add this component to all selected entities
                                for e in &all_entities {
                                    // e guaranteed to be Some, this is a new transaction and we aren't deleting entities
                                    component_type_to_add.add_default_to_entity(tx.world_mut(), *e)
                                }

                                commit_required |= true;
                            }

                            //
                            // Draw inspect widgets
                            //
                            let inspect_registry =
                                resources.get::<EditorInspectRegistryResource>().unwrap();
                            commit_required |= inspect_registry.render_mut(
                                tx.world_mut(),
                                &all_entities,
                                ui,
                                &Default::default(),
                            );

                            if commit_required {
                                tx.commit(
                                    &*asset_resource,
                                    &mut editor_ui_state,
                                    PostCommitSelection::KeepCurrentSelection,
                                    &*component_registry,
                                );
                            }
                        }
                    });
            }
        });
    }
}
