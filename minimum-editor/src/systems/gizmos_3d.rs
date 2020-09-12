use legion::*;

use minimum_game::resources::{InputResource, ViewportResource, DebugDraw3DResource};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, EditorDraw3DResource, EditorDraw3DConstraint,
    EditorTransaction, PostCommitSelection,
};
use crate::resources::EditorTool;

use minimum_game::input::MouseButton;

use minimum_transform::components::{TransformComponentDef, TransformComponent};

use legion::query::{EntityFilter, Query};
use legion::query::And;
use legion::query::ComponentFilter;
use legion::query::Passthrough;

use minimum_kernel::resources::ComponentRegistryResource;
use minimum_kernel::resources::AssetResource;
use minimum_game::resources::DebugDraw3DDepthBehavior;
use legion::world::SubWorld;

//TODO: Adapt the size of "hot" area around the editor drawn shapes based on zoom level

// The DefaultFilter is EntityFilterTuple<ComponentFilter<TransformComponent>, Passthrough>
type TransformQuery = Query<
    (Entity, Read<TransformComponent>),
    <(Entity, Read<TransformComponent>) as legion::query::DefaultFilter>::Filter,
>;

pub fn editor_gizmos(schedule: &mut legion::systems::Builder) {
    schedule.add_system(
        SystemBuilder::new("editor_input")
            .read_resource::<ViewportResource>()
            .write_resource::<EditorStateResource>()
            .read_resource::<InputResource>()
            .read_resource::<ViewportResource>()
            .write_resource::<EditorSelectionResource>()
            .write_resource::<DebugDraw3DResource>()
            .write_resource::<EditorDraw3DResource>()
            .read_resource::<ComponentRegistryResource>()
            .read_resource::<AssetResource>()
            .with_query(<(Entity, Read<TransformComponent>)>::query())
            .build(
                |_command_buffer,
                 subworld,
                 (
                    viewport_resource,
                    editor_state,
                    _input_state,
                    _viewport,
                    editor_selection,
                    debug_draw,
                    editor_draw,
                    component_registry,
                    asset_resource,
                ),
                 transform_query| {
                    let mut gizmo_tx = None;
                    std::mem::swap(&mut gizmo_tx, editor_state.gizmo_transaction_mut());

                    if gizmo_tx.is_none() {
                        gizmo_tx = editor_state.create_transaction_from_selected(
                            &*editor_selection,
                            &*component_registry,
                        );
                    }

                    if let Some(mut gizmo_tx) = gizmo_tx {
                        let mut result = GizmoResult::NoChange;
                        result = result.max(handle_translate_gizmo_input(
                            &mut *editor_draw,
                            &mut gizmo_tx,
                        ));
                        result =
                            result.max(handle_scale_gizmo_input(&mut *editor_draw, &mut gizmo_tx));
                        result =
                            result.max(handle_rotate_gizmo_input(&mut *editor_draw, &mut gizmo_tx));

                        match result {
                            GizmoResult::NoChange => {}
                            GizmoResult::Update => {
                                gizmo_tx.update(
                                    asset_resource,
                                    editor_state,
                                    PostCommitSelection::KeepCurrentSelection,
                                    &*component_registry,
                                );
                                *editor_state.gizmo_transaction_mut() = Some(gizmo_tx);
                            }
                            GizmoResult::Commit => {
                                gizmo_tx.commit(
                                    asset_resource,
                                    editor_state,
                                    PostCommitSelection::KeepCurrentSelection,
                                    &*component_registry,
                                );
                            }
                        }
                    }

                    match editor_state.active_editor_tool() {
                        EditorTool::Translate => draw_translate_gizmo(
                            &*viewport_resource,
                            &mut *debug_draw,
                            &mut *editor_draw,
                            &mut *editor_selection,
                            subworld,
                            transform_query,
                        ),
                        EditorTool::Scale => draw_scale_gizmo(
                            &*viewport_resource,
                            &mut *debug_draw,
                            &mut *editor_draw,
                            &mut *editor_selection,
                            subworld,
                            transform_query,
                        ),
                        EditorTool::Rotate => draw_rotate_gizmo(
                            &*viewport_resource,
                            &mut *debug_draw,
                            &mut *editor_draw,
                            &mut *editor_selection,
                            subworld,
                            transform_query,
                        ),
                    }
                },
            ),
    );
}

#[derive(Ord, PartialOrd, PartialEq, Eq)]
enum GizmoResult {
    NoChange,
    Update,
    Commit,
}

fn handle_translate_gizmo_input(
    editor_draw: &mut EditorDraw3DResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::LEFT)
    {
        // See what if any axis we will operate on
        let mut translate_x = false;
        let mut translate_y = false;
        let mut translate_z = false;
        if drag_in_progress.shape_id == "x_axis_translate" {
            translate_x = true;
        } else if drag_in_progress.shape_id == "y_axis_translate" {
            translate_y = true;
        } else if drag_in_progress.shape_id == "z_axis_translate" {
            translate_z = true;
        } else if drag_in_progress.shape_id == "xy_axis_translate" {
            translate_x = true;
            translate_y = true;
        } else if drag_in_progress.shape_id == "xz_axis_translate" {
            translate_x = true;
            translate_z = true;
        } else if drag_in_progress.shape_id == "yz_axis_translate" {
            translate_y = true;
            translate_z = true;
        }

        // Early out if we didn't touch either axis
        if !translate_x && !translate_y && !translate_z {
            return GizmoResult::NoChange;
        }

        // Determine the drag distance in ui_space
        let mut world_space_previous_frame_delta =
            drag_in_progress.world_space_previous_frame_delta;

        let mut query = <(Entity, Write<TransformComponentDef>)>::query();

        for (_entity_handle, mut position) in query.iter_mut(tx.world_mut()) {
            // Can use editor_draw.is_shape_drag_just_finished(MouseButton::LEFT) to see if this is the final drag,
            // in which case we might want to save an undo step
            *position.position += glam::Vec3::new(
                world_space_previous_frame_delta.x(),
                world_space_previous_frame_delta.y(),
                world_space_previous_frame_delta.z(),
            );
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::LEFT) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_translate_gizmo(
    viewport: &ViewportResource,
    debug_draw: &mut DebugDraw3DResource,
    editor_draw: &mut EditorDraw3DResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    transform_query: &mut TransformQuery,
) {
    for (entity, transform) in transform_query.iter(subworld) {
        if !selection_world.is_entity_selected(*entity) {
            continue;
        }

        let x_color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        let y_color = glam::vec4(0.0, 1.0, 0.0, 1.0);
        let z_color = glam::vec4(0.0, 0.0, 1.0, 1.0);
        let xy_color = glam::vec4(1.0, 1.0, 0.0, 1.0);
        let xz_color = glam::vec4(1.0, 0.0, 1.0, 1.0);
        let yz_color = glam::vec4(0.0, 1.0, 1.0, 1.0);

        let position = transform.position();
        let ui_multiplier = 0.005 * viewport.world_space_ui_multiplier(position);

        // x axis line
        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            position,
            position + (glam::vec3(100.0, 0.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::x_line(position),
            x_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            position + (glam::vec3(85.0, 15.0, 0.0) * ui_multiplier),
            position + (glam::vec3(100.0, 0.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::x_line(position),
            x_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "x_axis_translate",
            debug_draw,
            position + (glam::vec3(85.0, -15.0, 0.0) * ui_multiplier),
            position + (glam::vec3(100.0, 0.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::x_line(position),
            x_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // y axis line
        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            position,
            position + (glam::vec3(0.0, 100.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::y_line(position),
            y_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            position + (glam::vec3(-15.0, 85.0, 0.0) * ui_multiplier),
            position + (glam::vec3(0.0, 100.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::y_line(position),
            y_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "y_axis_translate",
            debug_draw,
            position + (glam::vec3(15.0, 85.0, 0.0) * ui_multiplier),
            position + (glam::vec3(0.0, 100.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::y_line(position),
            y_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // z axis line
        editor_draw.add_line(
            "z_axis_translate",
            debug_draw,
            position,
            position + (glam::vec3(0.0, 0.0, 100.0) * ui_multiplier),
            EditorDraw3DConstraint::z_line(position),
            z_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "z_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, -15.0, 85.0) * ui_multiplier),
            position + (glam::vec3(0.0, 0.0, 100.0) * ui_multiplier),
            EditorDraw3DConstraint::z_line(position),
            z_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "z_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, 15.0, 85.0) * ui_multiplier),
            position + (glam::vec3(0.0, 0.0, 100.0) * ui_multiplier),
            EditorDraw3DConstraint::z_line(position),
            z_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // xy line
        editor_draw.add_line(
            "xy_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, 25.0, 0.0) * ui_multiplier),
            position + (glam::vec3(25.0, 25.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::xy_plane(position),
            xy_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "xy_axis_translate",
            debug_draw,
            position + (glam::vec3(25.0, 0.0, 0.0) * ui_multiplier),
            position + (glam::vec3(25.0, 25.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::xy_plane(position),
            xy_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // xz line
        editor_draw.add_line(
            "xz_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, 0.0, 25.0) * ui_multiplier),
            position + (glam::vec3(25.0, 0.0, 25.0) * ui_multiplier),
            EditorDraw3DConstraint::xz_plane(position),
            xz_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "xz_axis_translate",
            debug_draw,
            position + (glam::vec3(25.0, 0.0, 0.0) * ui_multiplier),
            position + (glam::vec3(25.0, 0.0, 25.0) * ui_multiplier),
            EditorDraw3DConstraint::xz_plane(position),
            xz_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // yz line
        editor_draw.add_line(
            "yz_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, 0.0, 25.0) * ui_multiplier),
            position + (glam::vec3(0.0, 25.0, 25.0) * ui_multiplier),
            EditorDraw3DConstraint::yz_plane(position),
            yz_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        editor_draw.add_line(
            "yz_axis_translate",
            debug_draw,
            position + (glam::vec3(0.0, 25.0, 0.0) * ui_multiplier),
            position + (glam::vec3(0.0, 25.0, 25.0) * ui_multiplier),
            EditorDraw3DConstraint::yz_plane(position),
            yz_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );
    }
}

fn sign_aware_magnitude(v: glam::Vec3) -> f32 {
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

    total += if v.z() > 0.0 {
        v.z() * v.z()
    } else {
        v.z() * v.z() * -1.0
    };

    if total >= 0.0 {
        total.sqrt()
    } else {
        (total * -1.0).sqrt() * -1.0
    }
}

fn handle_scale_gizmo_input(
    editor_draw: &mut EditorDraw3DResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::LEFT)
    {
        // See what if any axis we will operate on
        let mut scale_x = false;
        let mut scale_y = false;
        let mut scale_z = false;
        let mut scale_uniform = false;
        if drag_in_progress.shape_id == "x_axis_scale" {
            scale_x = true;
        } else if drag_in_progress.shape_id == "y_axis_scale" {
            scale_y = true;
        } else if drag_in_progress.shape_id == "y_axis_scale" {
            scale_z = true;
        } else if drag_in_progress.shape_id == "uniform_scale" {
            scale_uniform = true;
        }

        // Early out if we didn't touch either axis
        if !scale_x && !scale_y && !scale_z && !scale_uniform {
            return GizmoResult::NoChange;
        }

        // Determine the drag distance
        let mut ui_space_previous_frame_delta = drag_in_progress.world_space_previous_frame_delta;

        if !scale_x && !scale_uniform {
            ui_space_previous_frame_delta.set_x(0.0);
        }

        if !scale_y && !scale_uniform {
            ui_space_previous_frame_delta.set_y(0.0);
        }

        if !scale_z && !scale_uniform {
            ui_space_previous_frame_delta.set_z(0.0);
        }

        // Pretty sure the sign_aware_magnitude is messing up the FP precision. Probably need to rethink
        // this as a UI-space circle around the basis
        if scale_uniform {
            let mag = sign_aware_magnitude(ui_space_previous_frame_delta);
            ui_space_previous_frame_delta.set_x(mag);
            ui_space_previous_frame_delta.set_y(mag);
            ui_space_previous_frame_delta.set_z(mag);
        }

        if scale_uniform {
            let mut query = <Write<TransformComponentDef>>::query();

            for mut transform in query.iter_mut(tx.world_mut()) {
                *transform.uniform_scale_mut() += ui_space_previous_frame_delta.x()
            }
        } else {
            let mut query = <Write<TransformComponentDef>>::query();

            for mut non_uniform_scale in query.iter_mut(tx.world_mut()) {
                *non_uniform_scale.non_uniform_scale += glam::Vec3::new(
                    ui_space_previous_frame_delta.x(),
                    ui_space_previous_frame_delta.y(),
                    ui_space_previous_frame_delta.z(),
                );
            }
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::LEFT) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_scale_gizmo(
    viewport: &ViewportResource,
    debug_draw: &mut DebugDraw3DResource,
    editor_draw: &mut EditorDraw3DResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    transform_query: &mut TransformQuery,
) {
    for (entity, transform) in transform_query.iter(subworld) {
        if !selection_world.is_entity_selected(*entity) {
            continue;
        }

        let position = transform.position();

        let x_color = glam::Vec4::new(0.0, 1.0, 0.0, 1.0);
        let y_color = glam::Vec4::new(1.0, 0.6, 0.0, 1.0);
        let xy_color = glam::Vec4::new(1.0, 1.0, 0.0, 1.0);

        let ui_multiplier = 0.005 * viewport.world_space_ui_multiplier(position);

        // x axis line
        editor_draw.add_line(
            "x_axis_scale",
            debug_draw,
            position,
            position + (glam::vec3(100.0, 0.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::x_line(position),
            x_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // x axis line end
        editor_draw.add_line(
            "x_axis_scale",
            debug_draw,
            position + (glam::vec3(100.0, -20.0, 0.0) * ui_multiplier),
            position + (glam::vec3(100.0, 20.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::x_line(position),
            x_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // y axis line
        editor_draw.add_line(
            "y_axis_scale",
            debug_draw,
            position,
            position + (glam::vec3(0.0, 100.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::y_line(position),
            y_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // y axis line end
        editor_draw.add_line(
            "y_axis_scale",
            debug_draw,
            position + (glam::Vec3::new(-20.0, 100.0, 0.0) * ui_multiplier),
            position + (glam::Vec3::new(20.0, 100.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::y_line(position),
            y_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // xy line
        editor_draw.add_line(
            "uniform_scale",
            debug_draw,
            position + (glam::Vec3::new(0.0, 0.0, 0.0) * ui_multiplier),
            position + (glam::Vec3::new(50.0, 50.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::xy_plane(position),
            xy_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );

        // xy line
        editor_draw.add_line(
            "uniform_scale",
            debug_draw,
            position + (glam::Vec3::new(40.0, 60.0, 0.0) * ui_multiplier),
            position + (glam::Vec3::new(60.0, 40.0, 0.0) * ui_multiplier),
            EditorDraw3DConstraint::xy_plane(position),
            xy_color,
            DebugDraw3DDepthBehavior::NoDepthTest,
        );
    }
}

fn handle_rotate_gizmo_input(
    editor_draw: &mut EditorDraw3DResource,
    tx: &mut EditorTransaction,
) -> GizmoResult {
    if let Some(drag_in_progress) =
        editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::LEFT)
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

        let mut query = <(Entity, Write<TransformComponentDef>)>::query();
        for (_entity_handle, mut rotation) in query.iter_mut(tx.world_mut()) {
            *rotation.rotation_euler_mut() += glam::Vec3::unit_z() * ui_space_previous_frame_delta
        }

        if editor_draw.is_shape_drag_just_finished(MouseButton::LEFT) {
            GizmoResult::Commit
        } else {
            GizmoResult::Update
        }
    } else {
        GizmoResult::NoChange
    }
}

fn draw_rotate_gizmo(
    viewport: &ViewportResource,
    debug_draw: &mut DebugDraw3DResource,
    editor_draw: &mut EditorDraw3DResource,
    selection_world: &mut EditorSelectionResource,
    subworld: &SubWorld,
    transform_query: &mut TransformQuery,
) {
    for (entity, transform) in transform_query.iter(subworld) {
        if !selection_world.is_entity_selected(*entity) {
            continue;
        }

        let position = transform.position();

        let z_axis_color = glam::Vec4::new(0.0, 1.0, 0.0, 1.0);

        let ui_multiplier = 0.005 * viewport.world_space_ui_multiplier(position);

        // editor_draw.add_sphere(
        //     "z_axis_rotate",
        //     debug_draw,
        //     *position,
        //     50.0 * ui_multiplier,
        //     12,
        //     z_axis_color,
        //     DebugDraw3DDepthBehavior::NoDepthTest,
        // );
        // editor_draw.add_sphere(
        //     "z_axis_rotate",
        //     debug_draw,
        //     *position,
        //     52.0 * ui_multiplier,
        //     12,
        //     z_axis_color,
        //     DebugDraw3DDepthBehavior::NoDepthTest,
        // );
    }
}
