use legion::*;

use minimum_game::resources::{InputResource};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, PostCommitSelection, EditorSettingsResource,
};
use minimum_game::resources::ImguiResource;

use imgui::im_str;

use minimum_kernel::resources::ComponentRegistryResource;
use minimum_kernel::resources::AssetResource;
use crate::components::EditorMetadataComponent;

pub fn editor_entity_list_window(schedule: &mut legion::systems::Builder) {
    
    schedule.add_system(
        SystemBuilder::new("editor_entity_list_window")
            .write_resource::<ImguiResource>()
            .write_resource::<EditorStateResource>()
            .write_resource::<EditorSelectionResource>()
            .read_resource::<InputResource>()
            .read_resource::<ComponentRegistryResource>()
            .read_resource::<EditorSettingsResource>()
            .read_resource::<AssetResource>()
            .with_query(<(Entity, TryRead<EditorMetadataComponent>)>::query())
            .build(
                |_,
                 world,
                 (
                    imgui_manager,
                    editor_ui_state,
                    editor_selection,
                    input,
                    component_registry,
                    editor_settings,
                    asset_resource,
                ),
                 all_query| {
                    imgui_manager.with_ui(|ui: &mut imgui::Ui| {
                        let window_options = editor_ui_state.window_options();

                        if window_options.show_entity_list {
                            imgui::Window::new(im_str!("Entity List"))
                                .position([0.0, 50.0], imgui::Condition::Once)
                                .size([350.0, 250.0], imgui::Condition::Once)
                                .build(ui, || {
                                    let add_entity =
                                        ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]);
                                    ui.same_line_with_spacing(80.0, 10.0);
                                    let remove_entity =
                                        ui.button(im_str!("\u{e897} Delete"), [80.0, 0.0]);

                                    if add_entity {
                                        //TODO: Update selection
                                        if let Some(mut tx) = editor_ui_state
                                            .create_empty_transaction(&*component_registry)
                                        {
                                            tx.world_mut().push(());
                                            tx.commit(
                                                &*asset_resource,
                                                &mut *editor_ui_state,
                                                PostCommitSelection::SelectAllInTransaction,
                                                &*component_registry,
                                            );
                                        }
                                    }

                                    if remove_entity {
                                        if let Some(mut tx) = editor_ui_state
                                            .create_transaction_from_selected(
                                                &*editor_selection,
                                                &*component_registry,
                                            )
                                        {
                                            tx.world_mut().clear();
                                            tx.commit(
                                                &*asset_resource,
                                                &mut *editor_ui_state,
                                                PostCommitSelection::KeepCurrentSelection,
                                                &*component_registry,
                                            );
                                        }
                                    }

                                    let name = im_str!("");
                                    if unsafe {
                                        imgui::sys::igListBoxHeaderVec2(
                                            name.as_ptr(),
                                            imgui::sys::ImVec2 { x: -1.0, y: -1.0 },
                                        )
                                    } {
                                        for (e, editor_metadata) in all_query.iter(world) {
                                            let is_selected =
                                                editor_selection.is_entity_selected(*e);

                                            let s = if let Some(editor_metadata) = editor_metadata {
                                                im_str!("{:?}", editor_metadata.name)
                                            } else {
                                                im_str!("{:?}", e)
                                            };

                                            let clicked = imgui::Selectable::new(&s)
                                                .selected(is_selected)
                                                .build(ui);

                                            if clicked {
                                                //TODO: Hook up keyboard controls
                                                let is_control_held = input.is_key_down(
                                                    editor_settings.keybinds().selection_toggle,
                                                );
                                                if is_control_held {
                                                    if !is_selected {
                                                        // Add this entity
                                                        editor_selection
                                                            .enqueue_add_to_selection(vec![*e]);
                                                    } else {
                                                        //Remove this entity
                                                        editor_selection
                                                            .enqueue_remove_from_selection(vec![
                                                                *e,
                                                            ]);
                                                    }
                                                } else {
                                                    // Select just this entity
                                                    editor_selection
                                                        .enqueue_set_selection(vec![*e]);
                                                }
                                            }
                                        }

                                        unsafe {
                                            imgui::sys::igListBoxFooter();
                                        }
                                    }
                                });
                        }
                    })
                },
            ),
    );
}
