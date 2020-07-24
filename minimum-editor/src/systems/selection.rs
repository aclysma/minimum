use legion::prelude::*;

use minimum_game::resources::{InputResource, ViewportResource, DebugDraw2DResource, DebugDraw3DResource, DebugDraw3DDepthBehavior, UniverseResource};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, EditorDraw3DResource, EditorSettingsResource,
};

use minimum_game::input::MouseButton;

use ncollide3d::pipeline::{CollisionGroups};

use minimum_transform::components::TransformComponentDef;
use ncollide3d::query::RayIntersection;
use ncollide3d::shape::{ConvexHull, Shape};
use ncollide3d::query::algorithms::gjk::GJKResult::Proximity;
use minimum_math::na_convert::vec3_glam_to_glm;

fn handle_selection(
    editor_draw: &EditorDraw3DResource,
    input_state: &InputResource,
    viewport: &ViewportResource,
    editor_selection: &mut EditorSelectionResource,
    debug_draw_3d: &mut DebugDraw3DResource,
    debug_draw_2d: &mut DebugDraw2DResource,
    editor_settings: &EditorSettingsResource,
) {
    let mut intersecting_entities = None;

    if editor_draw.is_interacting_with_anything() {
        //
        // If the user is doing something with the editor draw API, disable the selection logic
        //
    }
    else if let Some(position) = input_state.mouse_button_just_clicked_position(MouseButton::LEFT)
    {
        //
        // Handle a single click. Do a raycast to find find anything under the mouse position
        //
        let ray = viewport.viewport_space_to_ray(position);
        trace!("Single click selection raycast: {:?}", ray);

        // Convert to nalgebra
        let nray = ncollide3d::query::Ray::new(
            ncollide3d::math::Point::from(vec3_glam_to_glm(ray.origin)),
            ncollide3d::math::Vector::from(vec3_glam_to_glm(ray.dir))
        );

        // Do the raycast
        let collision_groups = CollisionGroups::default();
        let results = editor_selection
            .editor_selection_world()
            .interferences_with_ray(&nray, &collision_groups);

        // Find all the entities that were hit and set the selected entity set to those entities
        let results: Vec<Entity> = results.map(|(_, x, _)| *x.data()).collect();
        intersecting_entities = Some(results);
    } else if let Some(drag_complete) = input_state.mouse_drag_just_finished(MouseButton::LEFT) {
        //
        // Handle user finishing dragging a box around entities. Create a shape that matches the
        // drag location in the world and project it into space to find intersecting entities
        //
        let p0 = drag_complete.begin_position;
        let p2 = drag_complete.end_position;
        let p1 = glam::Vec2::new(p0.x(), p2.y());
        let p3 = glam::Vec2::new(p2.x(), p0.y());

        let p0_segment = viewport.viewport_space_to_segment(p0);
        let p1_segment = viewport.viewport_space_to_segment(p1);
        let p2_segment = viewport.viewport_space_to_segment(p2);
        let p3_segment = viewport.viewport_space_to_segment(p3);

        let points = vec![
            ncollide3d::math::Point::from(vec3_glam_to_glm(p0_segment.p0)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p0_segment.p1)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p1_segment.p0)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p1_segment.p1)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p2_segment.p0)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p2_segment.p1)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p3_segment.p0)),
            ncollide3d::math::Point::from(vec3_glam_to_glm(p3_segment.p1)),
        ];
        if let Some(convex_shape) = ConvexHull::try_from_points(&points) {

            let collision_groups = CollisionGroups::default();
            let aabb = convex_shape.aabb(&ncollide3d::math::Isometry::identity());
            let interferences: Vec<_> = editor_selection.editor_selection_world().interferences_with_aabb(&aabb, &collision_groups).collect();

            let results = interferences.into_iter().filter_map(move |(handle, x)| {
                let prox = ncollide3d::query::proximity(
                    &ncollide3d::math::Isometry::identity(),
                    &convex_shape,
                    x.position(),
                    x.shape().as_ref(),
                    0.0,
                );

                if prox == ncollide3d::query::Proximity::Intersecting {
                    Some(*x.data())
                } else {
                    None
                }
            }).collect();
            intersecting_entities = Some(results);
        } else {
            intersecting_entities = None;
        }
    } else if let Some(drag_in_progress) = input_state.mouse_drag_in_progress(MouseButton::LEFT) {
        //
        // User is dragging a box around entities. Just draw the box.
        //
        //TODO: use 2d debug draw
        let p0 = drag_in_progress.begin_position;
        let p2 = drag_in_progress.end_position;
        let p1 = glam::Vec2::new(p0.x(), p2.y());
        let p3 = glam::Vec2::new(p2.x(), p0.y());

        let points_2d = vec![
            p0,
            p1,
            p2,
            p3,
        ];

        debug_draw_2d.add_line_loop(
            points_2d,
            glam::vec4(1.0, 1.0, 0.0, 1.0),
        );

        // let points_3d = vec![
        //     viewport.viewport_space_to_world_space(p0, 0.01),
        //     viewport.viewport_space_to_world_space(p1, 0.01),
        //     viewport.viewport_space_to_world_space(p2, 0.01),
        //     viewport.viewport_space_to_world_space(p3, 0.01),
        // ];
        //
        // debug_draw_3d.add_line_loop(
        //     points_3d,
        //     glam::vec4(1.0, 1.0, 0.0, 1.0),
        //     DebugDraw3DDepthBehavior::NoDepthTest
        // );
    }

    if let Some(intersecting_entities) = intersecting_entities {
        let add_to_selection = input_state.is_key_down(editor_settings.keybinds().selection_add);
        let subtract_from_selection =
            input_state.is_key_down(editor_settings.keybinds().selection_subtract);
        let toggle_selection = input_state.is_key_down(editor_settings.keybinds().selection_toggle);

        let mut any_not_selected = false;
        for e in &intersecting_entities {
            if !editor_selection.selected_entities().contains(e) {
                any_not_selected = true;
                break;
            }
        }

        let is_drag = input_state
            .mouse_drag_just_finished(MouseButton::LEFT)
            .is_some();

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
        .write_resource::<DebugDraw3DResource>()
        .write_resource::<DebugDraw2DResource>()
        .write_resource::<EditorDraw3DResource>()
        .read_resource::<UniverseResource>()
        .read_resource::<EditorSettingsResource>()
        .build(
            |_command_buffer,
             _subworld,
             (
                _editor_state,
                input_state,
                viewport,
                editor_selection,
                debug_draw_3d,
                debug_draw_2d,
                editor_draw,
                _universe_resource,
                editor_settings,
             ),
             _| {
                handle_selection(
                    &*editor_draw,
                    &*input_state,
                    &*viewport,
                    &mut *editor_selection,
                    &mut *debug_draw_3d,
                    &mut *debug_draw_2d,
                    &*editor_settings,
                );
            },
        )
}

pub fn draw_selection_shapes() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_selection_shapes")
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDraw3DResource>()
        .build(|_, _, (editor_selection, debug_draw), _| {
            let aabbs = editor_selection.selected_entity_aabbs();

            for (_, aabb) in aabbs {
                if let Some(aabb) = aabb {
                    let color = glam::vec4(1.0, 1.0, 0.0, 1.0);

                    // An amount to expand the AABB by so that we don't draw on top of the shape.
                    // Found in actual usage this ended up being annoying.
                    let expand = glam::vec2(0.0, 0.0);

                    let aabb = minimum_math::BoundingAabb {
                        min: glam::Vec3::new(aabb.mins().x, aabb.mins().y, aabb.mins().z),
                        max: glam::Vec3::new(aabb.maxs().x, aabb.maxs().y, aabb.maxs().z)
                    };

                    debug_draw.add_aabb(aabb, color, DebugDraw3DDepthBehavior::NoDepthTest);
                }
            }
        })
}
