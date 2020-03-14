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
use crate::pipeline::PrefabAsset;
use prefab_format::{EntityUuid, ComponentTypeUuid};
use legion_prefab::CookedPrefab;
use legion_transaction::ComponentDiff;
use std::sync::Arc;
use crate::components::{Position2DComponent, Rotation2DComponent};
use crate::components::UniformScale2DComponent;
use crate::components::NonUniformScale2DComponent;
use atelier_assets::core::asset_uuid;

use legion::filter::EntityFilterTuple;
use legion::filter::And;
use legion::filter::ComponentFilter;
use legion::filter::Passthrough;
use legion::systems::SystemQuery;
use legion::systems::SubWorld;

//TODO: Adapt the size of "hot" area around the editor drawn shapes based on zoom level

pub fn editor_gizmos() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_input")
        .write_resource::<EditorStateResource>()
        .read_resource::<InputResource>()
        .read_resource::<ViewportResource>()
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDrawResource>()
        .write_resource::<EditorDrawResource>()
        .read_resource::<UniverseResource>()
        .with_query(<(Read<Position2DComponent>)>::query())
        .with_query(<(
            Read<Position2DComponent>,
            TryRead<UniformScale2DComponent>,
            TryRead<NonUniformScale2DComponent>,
        )>::query())
        .with_query(<(Read<Position2DComponent>, Read<Rotation2DComponent>)>::query())
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
             (translate_query, scale_query, rotate_query)| {
                let mut gizmo_tx = None;
                std::mem::swap(&mut gizmo_tx, editor_state.gizmo_transaction_mut());

                if gizmo_tx.is_none() {
                    gizmo_tx = editor_state
                        .create_transaction_from_selected(&*editor_selection, &*universe_resource);
                }

                if let Some(mut gizmo_tx) = gizmo_tx {
                    let mut result = GizmoResult::NoChange;
                    result = result.max(handle_translate_gizmo_input(
                        &mut *editor_draw,
                        &mut gizmo_tx,
                    ));
                    result = result.max(handle_scale_gizmo_input(&mut *editor_draw, &mut gizmo_tx));
                    result =
                        result.max(handle_rotate_gizmo_input(&mut *editor_draw, &mut gizmo_tx));

                    match result {
                        GizmoResult::NoChange => {}
                        GizmoResult::Update => {
                            gizmo_tx
                                .update(editor_state, PostCommitSelection::KeepCurrentSelection);
                            *editor_state.gizmo_transaction_mut() = Some(gizmo_tx);
                        }
                        GizmoResult::Commit => {
                            gizmo_tx
                                .commit(editor_state, PostCommitSelection::KeepCurrentSelection);
                        }
                    }
                }

                match editor_state.active_editor_tool() {
                    EditorTool::Translate => draw_translate_gizmo(
                        &mut *debug_draw,
                        &mut *editor_draw,
                        &mut *editor_selection,
                        subworld,
                        translate_query,
                    ),
                    EditorTool::Scale => draw_scale_gizmo(
                        &mut *debug_draw,
                        &mut *editor_draw,
                        &mut *editor_selection,
                        subworld,
                        scale_query,
                    ),
                    EditorTool::Rotate => draw_rotate_gizmo(
                        &mut *debug_draw,
                        &mut *editor_draw,
                        &mut *editor_selection,
                        subworld,
                        rotate_query,
                    ),
                    _ => {}
                }
            },
        )
}

#[derive(Ord, PartialOrd, PartialEq, Eq)]
enum GizmoResult {
    NoChange,
    Update,
    Commit,
}

fn handle_translate_gizmo_input(
    editor_draw: &mut EditorDrawResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left)
    {
        // See what if any axis we will operate on
        let mut translate_x = false;
        let mut translate_y = false;
        if drag_in_progress.shape_id == "x_axis_translate" {
            translate_x = true;
        } else if drag_in_progress.shape_id == "y_axis_translate" {
            translate_y = true;
        } else if drag_in_progress.shape_id == "xy_axis_translate" {
            translate_x = true;
            translate_y = true;
        }

        // Early out if we didn't touch either axis
        if !translate_x && !translate_y {
            return GizmoResult::NoChange;
        }

        // Determine the drag distance in ui_space
        let mut world_space_previous_frame_delta =
            drag_in_progress.world_space_previous_frame_delta;

        if !translate_x {
            world_space_previous_frame_delta.set_x(0.0);
        }

        if !translate_y {
            world_space_previous_frame_delta.set_y(0.0);
        }

        let query = <(Write<Position2DComponent>)>::query();

        for (entity_handle, mut position) in query.iter_entities_mut(tx.world_mut()) {
            // Can use editor_draw.is_shape_drag_just_finished(MouseButton::Left) to see if this is the final drag,
            // in which case we might want to save an undo step
            *position.position += world_space_previous_frame_delta;
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_translate_gizmo(
    debug_draw: &mut DebugDrawResource,
    editor_draw: &mut EditorDrawResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    translate_query: &mut legion::systems::SystemQuery<
        Read<Position2DComponent>,
        EntityFilterTuple<ComponentFilter<Position2DComponent>, Passthrough, Passthrough>,
    >,
) {
    for (entity, position) in translate_query.iter_entities(subworld) {
        if !selection_world.is_entity_selected(entity) {
            continue;
        }

        let x_color = glam::vec4(0.0, 1.0, 0.0, 1.0);
        let y_color = glam::vec4(1.0, 0.6, 0.0, 1.0);
        let xy_color = glam::vec4(1.0, 1.0, 0.0, 1.0);

        let xy_position = glam::Vec2::new(position.position.x(), position.position.y());

        //TODO: Make this resolution independent. Need a UI multiplier?

        let ui_multiplier = 0.01;

        // x axis line
        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            xy_position,
            xy_position + (glam::vec2(100.0, 0.0) * ui_multiplier),
            x_color,
        );

        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(85.0, 15.0) * ui_multiplier),
            xy_position + (glam::vec2(100.0, 0.0) * ui_multiplier),
            x_color,
        );

        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(85.0, -15.0) * ui_multiplier),
            xy_position + (glam::vec2(100.0, 0.0) * ui_multiplier),
            x_color,
        );

        // y axis line
        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            xy_position,
            xy_position + (glam::vec2(0.0, 100.0) * ui_multiplier),
            y_color,
        );

        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(-15.0, 85.0) * ui_multiplier),
            xy_position + (glam::vec2(0.0, 100.0) * ui_multiplier),
            y_color,
        );

        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(15.0, 85.0) * ui_multiplier),
            xy_position + (glam::vec2(0.0, 100.0) * ui_multiplier),
            y_color,
        );

        // xy line
        editor_draw.add_line(
            "xy_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(0.0, 25.0) * ui_multiplier),
            xy_position + (glam::vec2(25.0, 25.0) * ui_multiplier),
            xy_color,
        );

        // xy line
        editor_draw.add_line(
            "xy_axis_translate",
            debug_draw,
            xy_position + (glam::vec2(25.0, 0.0) * ui_multiplier),
            xy_position + (glam::vec2(25.0, 25.0) * ui_multiplier),
            xy_color,
        );
    }
}

fn sign_aware_magnitude(v: glam::Vec2) -> f32 {
    let mut total = 0.0;
    total += if v.x() > 0.0 {
        v.x() * v.x()
    } else {
        v.x() * v.x() * -1.0
    };

    total += if v.y() > 0.0 {
        v.y() * v.y()
    } else {
        v.y() * v.y() * -1.0
    };

    if total >= 0.0 {
        total.sqrt()
    } else {
        (total * -1.0).sqrt() * -1.0
    }
}

fn handle_scale_gizmo_input(
    editor_draw: &mut EditorDrawResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left)
    {
        // See what if any axis we will operate on
        let mut scale_x = false;
        let mut scale_y = false;
        let mut scale_uniform = false;
        if drag_in_progress.shape_id == "x_axis_scale" {
            scale_x = true;
        } else if drag_in_progress.shape_id == "y_axis_scale" {
            scale_y = true;
        } else if drag_in_progress.shape_id == "uniform_scale" {
            scale_uniform = true;
        }

        // Early out if we didn't touch either axis
        if !scale_x && !scale_y && !scale_uniform {
            return GizmoResult::NoChange;
        }

        // Determine the drag distance in ui_space
        //TODO: I was intending this to use ui space but the values during drag are not lining up
        // with values on end drag. This is likely an fp precision issue.
        let mut ui_space_previous_frame_delta = drag_in_progress.world_space_previous_frame_delta;

        if !scale_x && !scale_uniform {
            ui_space_previous_frame_delta.set_x(0.0);
        }

        if !scale_y && !scale_uniform {
            ui_space_previous_frame_delta.set_y(0.0);
        }

        if scale_uniform {
            ui_space_previous_frame_delta
                .set_x(sign_aware_magnitude(ui_space_previous_frame_delta));
            ui_space_previous_frame_delta.set_y(ui_space_previous_frame_delta.x());
        }

        if scale_uniform {
            let query = <(Write<UniformScale2DComponent>)>::query();

            for (entity_handle, mut uniform_scale) in query.iter_entities_mut(tx.world_mut()) {
                uniform_scale.uniform_scale += ui_space_previous_frame_delta.x()
            }
        } else {
            let query = <(Write<NonUniformScale2DComponent>)>::query();

            for (entity_handle, mut non_uniform_scale) in query.iter_entities_mut(tx.world_mut()) {
                *non_uniform_scale.non_uniform_scale += ui_space_previous_frame_delta
            }
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_scale_gizmo(
    debug_draw: &mut DebugDrawResource,
    editor_draw: &mut EditorDrawResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    scale_query: &mut legion::systems::SystemQuery<
        (
            Read<Position2DComponent>,
            TryRead<UniformScale2DComponent>,
            TryRead<NonUniformScale2DComponent>,
        ),
        EntityFilterTuple<
            And<(
                ComponentFilter<Position2DComponent>,
                Passthrough,
                Passthrough,
            )>,
            And<(Passthrough, Passthrough, Passthrough)>,
            And<(Passthrough, Passthrough, Passthrough)>,
        >,
    >,
) {
    for (entity, (position, uniform_scale, non_uniform_scale)) in
        scale_query.iter_entities(subworld)
    {
        if !selection_world.is_entity_selected(entity) {
            continue;
        }

        let position = *position.position;

        let x_color = glam::Vec4::new(0.0, 1.0, 0.0, 1.0);
        let y_color = glam::Vec4::new(1.0, 0.6, 0.0, 1.0);
        let xy_color = glam::Vec4::new(1.0, 1.0, 0.0, 1.0);

        //TODO: Make this resolution independent. Need a UI multiplier?

        let ui_multiplier = 0.01;

        if non_uniform_scale.is_some() {
            // x axis line
            editor_draw.add_line(
                "x_axis_scale",
                debug_draw,
                position,
                position + (glam::vec2(100.0, 0.0) * ui_multiplier),
                x_color,
            );

            // x axis line end
            editor_draw.add_line(
                "x_axis_scale",
                debug_draw,
                position + (glam::vec2(100.0, -20.0) * ui_multiplier),
                position + (glam::vec2(100.0, 20.0) * ui_multiplier),
                x_color,
            );

            // y axis line
            editor_draw.add_line(
                "y_axis_scale",
                debug_draw,
                position,
                position + (glam::vec2(0.0, 100.0) * ui_multiplier),
                y_color,
            );

            // y axis line end
            editor_draw.add_line(
                "y_axis_scale",
                debug_draw,
                position + (glam::Vec2::new(-20.0, 100.0) * ui_multiplier),
                position + (glam::Vec2::new(20.0, 100.0) * ui_multiplier),
                y_color,
            );
        }

        if uniform_scale.is_some() {
            // xy line
            editor_draw.add_line(
                "uniform_scale",
                debug_draw,
                position + (glam::Vec2::new(0.0, 0.0) * ui_multiplier),
                position + (glam::Vec2::new(50.0, 50.0) * ui_multiplier),
                xy_color,
            );

            // xy line
            editor_draw.add_line(
                "uniform_scale",
                debug_draw,
                position + (glam::Vec2::new(40.0, 60.0) * ui_multiplier),
                position + (glam::Vec2::new(60.0, 40.0) * ui_multiplier),
                xy_color,
            );
        }
    }
}

fn handle_rotate_gizmo_input(
    editor_draw: &mut EditorDrawResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left)
    {
        // See what if any axis we will operate on
        let mut rotate_z = false;
        if drag_in_progress.shape_id == "z_axis_rotate" {
            rotate_z = true;
        }

        // Early out if we didn't touch either axis
        if !rotate_z {
            return GizmoResult::NoChange;
        }

        //TODO: It might be possible to detect the dragged shape's center, compare it to mouse
        // position, and track a 1:1 rotation with mouse movement

        // Determine the drag distance in ui_space
        //TODO: I was intending this to use ui space but the values during drag are not lining up
        // with values on end drag. This is likely an fp precision issue.
        let ui_space_previous_frame_delta =
            sign_aware_magnitude(drag_in_progress.world_space_previous_frame_delta);

        let query = <(Write<Rotation2DComponent>)>::query();
        for (entity_handle, mut rotation) in query.iter_entities_mut(tx.world_mut()) {
            rotation.rotation += ui_space_previous_frame_delta
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_rotate_gizmo(
    debug_draw: &mut DebugDrawResource,
    editor_draw: &mut EditorDrawResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    scale_query: &mut legion::systems::SystemQuery<
        (Read<Position2DComponent>, Read<Rotation2DComponent>),
        EntityFilterTuple<
            And<(
                ComponentFilter<Position2DComponent>,
                ComponentFilter<Rotation2DComponent>,
            )>,
            And<(Passthrough, Passthrough)>,
            And<(Passthrough, Passthrough)>,
        >,
    >,
) {
    for (entity, (position, rotation)) in scale_query.iter_entities(subworld) {
        if !selection_world.is_entity_selected(entity) {
            continue;
        }

        let position = *position.position;

        let z_axis_color = glam::Vec4::new(0.0, 1.0, 0.0, 1.0);

        //TODO: Make this resolution independent. Need a UI multiplier?
        let ui_multiplier = 0.01;

        editor_draw.add_circle_outline(
            "z_axis_rotate",
            debug_draw,
            position,
            50.0 * ui_multiplier,
            z_axis_color,
        );
        editor_draw.add_circle_outline(
            "z_axis_rotate",
            debug_draw,
            position,
            52.0 * ui_multiplier,
            z_axis_color,
        );
    }
}
