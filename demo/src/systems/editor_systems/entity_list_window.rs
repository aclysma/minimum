use legion::prelude::*;

use crate::resources::{
    EditorStateResource, InputResource, TimeResource, EditorSelectionResource, ViewportResource,
    DebugDrawResource, UniverseResource, EditorDrawResource, EditorTransaction,
    PostCommitSelection,
};
use crate::resources::ImguiResource;
use crate::resources::EditorTool;
use legion_transaction::{TransactionBuilder, Transaction};

use imgui;
use skulpin::app::VirtualKeyCode;
use skulpin::app::MouseButton;
use skulpin::LogicalSize;
use imgui::im_str;
use ncollide2d::pipeline::{CollisionGroups, CollisionObjectRef};

use std::collections::HashMap;
use ncollide2d::bounding_volume::AABB;
use ncollide2d::world::CollisionWorld;

use imgui_inspect_derive::Inspect;

use crate::math::winit_position_to_glam;
use imgui_inspect::InspectRenderDefault;
use minimum2::pipeline::PrefabAsset;
use prefab_format::{EntityUuid, ComponentTypeUuid};
use legion_prefab::CookedPrefab;
use legion_transaction::ComponentDiff;
use std::sync::Arc;
use crate::components::PositionComponent;
use atelier_assets::core::asset_uuid;

pub fn editor_entity_list_window() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_entity_list_window")
        .write_resource::<ImguiResource>()
        .write_resource::<EditorStateResource>()
        .write_resource::<EditorSelectionResource>()
        .read_resource::<InputResource>()
        .read_resource::<UniverseResource>()
        .with_query(<(TryRead<()>)>::query())
        .build(
            |_,
             world,
             (imgui_manager, editor_ui_state, editor_selection, input, universe_resource),
             all_query| {
                imgui_manager.with_ui(|ui: &mut imgui::Ui| {
                    use imgui::im_str;

                    let window_options = editor_ui_state.window_options();

                    if window_options.show_entity_list {
                        imgui::Window::new(im_str!("Entity List"))
                            .position([0.0, 50.0], imgui::Condition::Once)
                            .size([350.0, 250.0], imgui::Condition::Once)
                            .build(ui, || {
                                let add_entity = ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]);
                                ui.same_line_with_spacing(80.0, 10.0);
                                let remove_entity =
                                    ui.button(im_str!("\u{e897} Delete"), [80.0, 0.0]);

                                if add_entity {
                                    //TODO: Update selection
                                    if let Some(mut tx) = editor_ui_state
                                        .create_empty_transaction(&*universe_resource)
                                    {
                                        tx.world_mut().insert((), vec![()]);
                                        tx.commit(
                                            &mut *editor_ui_state,
                                            PostCommitSelection::SelectAllInTransaction,
                                        );
                                    }
                                }

                                if remove_entity {
                                    if let Some(mut tx) = editor_ui_state
                                        .create_transaction_from_selected(
                                            &*editor_selection,
                                            &*universe_resource,
                                        )
                                    {
                                        tx.world_mut().delete_all();
                                        tx.commit(
                                            &mut *editor_ui_state,
                                            PostCommitSelection::KeepCurrentSelection,
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
                                    for (e, _) in all_query.iter_entities(world) {
                                        let is_selected = editor_selection.is_entity_selected(e);

                                        let s = im_str!("{:?}", e);
                                        let clicked = imgui::Selectable::new(&s)
                                            .selected(is_selected)
                                            .build(ui);

                                        if clicked {
                                            let is_control_held = input
                                                .is_key_down(VirtualKeyCode::LControl)
                                                || input.is_key_down(VirtualKeyCode::RControl);
                                            if is_control_held {
                                                if !is_selected {
                                                    // Add this entity
                                                    editor_selection
                                                        .enqueue_add_to_selection(vec![e]);
                                                } else {
                                                    //Remove this entity
                                                    editor_selection
                                                        .enqueue_remove_from_selection(vec![e]);
                                                }
                                            } else {
                                                // Select just this entity
                                                editor_selection.enqueue_set_selection(vec![e]);
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
        )
}
