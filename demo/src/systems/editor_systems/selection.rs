use legion::prelude::*;

use crate::resources::{
    EditorStateResource, InputResource, TimeResource, EditorSelectionResource, ViewportResource,
    DebugDrawResource, UniverseResource, EditorDrawResource, EditorTransaction,
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

use crate::math_conversions::winit_position_to_glam;
use imgui_inspect::InspectRenderDefault;
use minimum2::pipeline::PrefabAsset;
use prefab_format::{EntityUuid, ComponentTypeUuid};
use legion_prefab::CookedPrefab;
use legion_transaction::ComponentDiff;
use std::sync::Arc;
use minimum2::components::PositionComponent;
use atelier_assets::core::asset_uuid;

fn handle_selection(
    editor_draw: &EditorDrawResource,
    input_state: &InputResource,
    viewport: &ViewportResource,
    editor_selection: &mut EditorSelectionResource,
    debug_draw: &mut DebugDrawResource,
) {
    let mut intersecting_entities = None;

    if editor_draw.is_interacting_with_anything() {
        println!("is interacting");
    //
    // If the user is doing something with the editor draw API, disable the selection logic
    //
    } else if let Some(position) = input_state.mouse_button_just_clicked_position(MouseButton::Left)
    {
        println!("just clicked");
        //
        // Handle a single click. Do a raycast to find find anything under the mouse position
        //

        // Determine where in world space to do the raycast
        let world_space = ncollide2d::math::Point::from(crate::math_conversions::vec2_glam_to_glm(
            viewport.ui_space_to_world_space(position),
        ));

        // Do the raycast
        let collision_groups = CollisionGroups::default();
        let results = editor_selection
            .editor_selection_world()
            .interferences_with_point(&world_space, &collision_groups);

        // Find all the entities that were hit and set the selected entity set to those entities
        let results: Vec<Entity> = results.map(|(_, x)| *x.data()).collect();
        intersecting_entities = Some(results);
    } else if let Some(drag_complete) = input_state.mouse_drag_just_finished(MouseButton::Left) {
        print!("DRAG COMPLETE");
        //
        // Handle user finishing dragging a box around entities. Create a shape that matches the
        // drag location in the world and project it into space to find intersecting entities
        //

        // Determine where in world space to do the intersection test
        let target_position0: glam::Vec2 = viewport
            .ui_space_to_world_space(drag_complete.begin_position)
            .into();
        let target_position1: glam::Vec2 = viewport
            .ui_space_to_world_space(drag_complete.end_position)
            .into();

        // Find the top-left corner
        let mins = glam::vec2(
            f32::min(target_position0.x(), target_position1.x()),
            f32::min(target_position0.y(), target_position1.y()),
        );

        // Find the bottom-right corner
        let maxs = glam::vec2(
            f32::max(target_position0.x(), target_position1.x()),
            f32::max(target_position0.y(), target_position1.y()),
        );

        // Build an AABB to use in the collision intersection test
        let aabb = ncollide2d::bounding_volume::AABB::new(
            nalgebra::Point::from(crate::math_conversions::vec2_glam_to_glm(mins)),
            nalgebra::Point::from(crate::math_conversions::vec2_glam_to_glm(maxs)),
        );

        // Do the intersection test
        let collision_groups = CollisionGroups::default();
        let results = editor_selection
            .editor_selection_world()
            .interferences_with_aabb(&aabb, &collision_groups);

        let results: Vec<Entity> = results.map(|(_, x)| *x.data()).collect();
        intersecting_entities = Some(results);
    } else if let Some(drag_in_progress) = input_state.mouse_drag_in_progress(MouseButton::Left) {
        //
        // User is dragging a box around entities. Just draw the box.
        //
        debug_draw.add_rect(
            viewport.ui_space_to_world_space(drag_in_progress.begin_position),
            viewport.ui_space_to_world_space(drag_in_progress.end_position),
            glam::vec4(1.0, 1.0, 0.0, 1.0),
        );
    }

    if let Some(intersecting_entities) = intersecting_entities {
        let add_to_selection = input_state.is_key_down(VirtualKeyCode::LShift)
            || input_state.is_key_down(VirtualKeyCode::RShift);
        let subtract_from_selection = input_state.is_key_down(VirtualKeyCode::LAlt)
            || input_state.is_key_down(VirtualKeyCode::RAlt);
        let toggle_selection = input_state.is_key_down(VirtualKeyCode::LControl)
            || input_state.is_key_down(VirtualKeyCode::RControl);

        let mut any_not_selected = false;
        for e in &intersecting_entities {
            if !editor_selection.selected_entities().contains(e) {
                any_not_selected = true;
                break;
            }
        }

        let is_drag = input_state
            .mouse_drag_just_finished(MouseButton::Left)
            .is_some();

        println!(
            "DRAG STATE {} {} {}",
            add_to_selection, subtract_from_selection, toggle_selection
        );

        if toggle_selection {
            // Only do toggling behavior for clicks. Box-dragging should only be additive
            if any_not_selected || is_drag {
                editor_selection.enqueue_add_to_selection(intersecting_entities);
            } else {
                editor_selection.enqueue_remove_from_selection(intersecting_entities);
            }
        } else if add_to_selection {
            editor_selection.enqueue_add_to_selection(intersecting_entities);
        } else if subtract_from_selection {
            editor_selection.enqueue_remove_from_selection(intersecting_entities);
        } else {
            editor_selection.enqueue_set_selection(intersecting_entities);
        }
    }
}

pub fn editor_handle_selection() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_input")
        .write_resource::<EditorStateResource>()
        .read_resource::<InputResource>()
        .read_resource::<ViewportResource>()
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDrawResource>()
        .write_resource::<EditorDrawResource>()
        .read_resource::<UniverseResource>()
        .with_query(<(Read<PositionComponent>)>::query())
        .build(
            |command_buffer,
             subworld,
             (
                editor_state,
                input_state,
                viewport,
                editor_selection,
                debug_draw,
                editor_draw,
                universe_resource,
            ),
             (position_query)| {
                handle_selection(
                    &*editor_draw,
                    &*input_state,
                    &*viewport,
                    &mut *editor_selection,
                    &mut *debug_draw,
                );
            },
        )
}

pub fn draw_selection_shapes() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_selection_shapes")
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDrawResource>()
        .build(|_, _, (editor_selection, debug_draw), _| {
            let aabbs = editor_selection.selected_entity_aabbs();

            for (_, aabb) in aabbs {
                if let Some(aabb) = aabb {
                    let color = glam::vec4(1.0, 1.0, 0.0, 1.0);

                    // An amount to expand the AABB by so that we don't draw on top of the shape.
                    // Found in actual usage this ended up being annoying.
                    let expand = glam::vec2(0.0, 0.0);

                    debug_draw.add_rect(
                        glam::vec2(aabb.mins().x, aabb.mins().y) - expand,
                        glam::vec2(aabb.maxs().x, aabb.maxs().y) + expand,
                        color,
                    );
                }
            }
        })
}
