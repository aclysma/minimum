use legion::prelude::*;

use minimum_game::resources::{
    InputResource, TimeResource, ViewportResource, DebugDrawResource, UniverseResource,
    ImguiResource,
};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, EditorDrawResource, EditorSettingsResource,
};

use minimum_game::input::MouseButton;

use ncollide2d::pipeline::{CollisionGroups};

use minimum_transform::components::PositionComponent;

pub fn vec2_glam_to_glm(value: glam::Vec2) -> glm::Vec2 {
    glm::Vec2::new(value.x(), value.y())
}

fn handle_selection(
    editor_draw: &EditorDrawResource,
    input_state: &InputResource,
    viewport: &ViewportResource,
    editor_selection: &mut EditorSelectionResource,
    debug_draw: &mut DebugDrawResource,
    editor_settings: &EditorSettingsResource,
) {
    let mut intersecting_entities = None;

    if editor_draw.is_interacting_with_anything() {
        //
        // If the user is doing something with the editor draw API, disable the selection logic
        //
    } else if let Some(position) = input_state.mouse_button_just_clicked_position(MouseButton::LEFT)
    {
        //
        // Handle a single click. Do a raycast to find find anything under the mouse position
        //

        // Determine where in world space to do the raycast
        let world_space = ncollide2d::math::Point::from(vec2_glam_to_glm(
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
    } else if let Some(drag_complete) = input_state.mouse_drag_just_finished(MouseButton::LEFT) {
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
            nalgebra::Point::from(vec2_glam_to_glm(mins)),
            nalgebra::Point::from(vec2_glam_to_glm(maxs)),
        );

        // Do the intersection test
        let collision_groups = CollisionGroups::default();
        let results = editor_selection
            .editor_selection_world()
            .interferences_with_aabb(&aabb, &collision_groups);

        let results: Vec<Entity> = results.map(|(_, x)| *x.data()).collect();
        intersecting_entities = Some(results);
    } else if let Some(drag_in_progress) = input_state.mouse_drag_in_progress(MouseButton::LEFT) {
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
        .write_resource::<DebugDrawResource>()
        .write_resource::<EditorDrawResource>()
        .read_resource::<UniverseResource>()
        .read_resource::<EditorSettingsResource>()
        .with_query(<Read<PositionComponent>>::query())
        .build(
            |_command_buffer,
             _subworld,
             (
                _editor_state,
                input_state,
                viewport,
                editor_selection,
                debug_draw,
                editor_draw,
                _universe_resource,
                editor_settings,
            ),
             _position_query| {
                handle_selection(
                    &*editor_draw,
                    &*input_state,
                    &*viewport,
                    &mut *editor_selection,
                    &mut *debug_draw,
                    &*editor_settings,
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
