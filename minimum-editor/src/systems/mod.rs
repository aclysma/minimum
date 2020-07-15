use legion::prelude::*;

use minimum_game::resources::{
    InputResource, ViewportResource, DebugDraw2DResource, DebugDraw3DResource, UniverseResource, CameraResource,
};
use crate::resources::{
    EditorStateResource, EditorSelectionResource, EditorDraw3DResource, EditorSettingsResource,
};
use crate::resources::EditorTool;

use minimum_game::input::MouseButton;

use minimum_transform::TransformComponent;

mod main_menu;
pub use main_menu::editor_imgui_menu;

mod entity_list_window;
pub use entity_list_window::editor_entity_list_window;

mod inspector_window;
pub use inspector_window::editor_inspector_window;

mod selection;
pub use selection::draw_selection_shapes;
pub use selection::editor_handle_selection;

// mod gizmos_2d;
// pub use gizmos_2d::editor_gizmos;

mod gizmos_3d;
pub use gizmos_3d::editor_gizmos;

pub fn editor_refresh_selection_world(
    world: &mut World,
    resources: &mut Resources,
) {
    let mut selection_world =
        EditorSelectionResource::create_editor_selection_world(resources, world);
    selection_world.update();
    resources
        .get_mut::<EditorSelectionResource>()
        .unwrap()
        .set_editor_selection_world(selection_world);
}

pub fn editor_process_selection_ops(
    world: &mut World,
    resources: &mut Resources,
) {
    let mut editor_selection = resources.get_mut::<EditorSelectionResource>().unwrap();
    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
    let universe = resources.get_mut::<UniverseResource>().unwrap();

    editor_selection.process_selection_ops(&mut *editor_state, &*universe, world);
}

pub fn reload_editor_state_if_file_changed(
    world: &mut World,
    resources: &mut Resources,
) {
    EditorStateResource::hot_reload_if_asset_changed(world, resources);
}

pub fn editor_process_edit_diffs(
    world: &mut World,
    resources: &mut Resources,
) {
    EditorStateResource::process_diffs(world, resources);
}

pub fn editor_process_editor_ops(
    world: &mut World,
    resources: &mut Resources,
) {
    EditorStateResource::process_editor_ops(world, resources);
}

pub fn editor_keybinds() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_input")
        .write_resource::<EditorStateResource>()
        .read_resource::<InputResource>()
        .read_resource::<ViewportResource>()
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDraw3DResource>()
        .write_resource::<EditorDraw3DResource>()
        .read_resource::<UniverseResource>()
        .read_resource::<EditorSettingsResource>()
        .build(
            |_command_buffer,
             _subworld,
             (
                editor_state,
                input_state,
                _viewport,
                _editor_selection,
                _debug_draw,
                _editor_draw,
                _universe_resource,
                editor_settings,
            ),
             _| {
                if input_state.is_key_just_down(editor_settings.keybinds().tool_translate) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Translate);
                }

                if input_state.is_key_just_down(editor_settings.keybinds().tool_scale) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Scale);
                }

                if input_state.is_key_just_down(editor_settings.keybinds().tool_rotate) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Rotate);
                }

                if input_state
                    .is_key_just_down(editor_settings.keybinds().action_toggle_editor_pause)
                {
                    editor_state.enqueue_toggle_pause();
                }
            },
        )
}

pub fn editor_mouse_input() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_input")
        .read_resource::<InputResource>()
        .write_resource::<CameraResource>()
        .read_resource::<ViewportResource>()
        .build(
            |_command_buffer, _subworld, (input_state, camera_resource, _viewport_resource), _| {
                // Right click drag pans the viewport
                // if let Some(mouse_drag) = input_state.mouse_drag_in_progress(MouseButton::RIGHT) {
                //     //let mut delta = mouse_drag.world_scale_previous_frame_delta;
                //     let mut delta = mouse_drag.ui_
                //     delta *= glam::Vec2::new(-1.0, -1.0);
                //     camera_resource.position += delta;
                // }

                // Right click drag pans the viewport
                let mouse_scroll = input_state.mouse_wheel_delta();
                let delta = mouse_scroll.y as f32;

                let delta = 1.05_f32.powf(-delta);
                camera_resource.x_half_extents *= delta;
            },
        )
}

pub fn editor_update_editor_draw() -> Box<dyn Schedulable> {
    SystemBuilder::new("editor_input")
        .write_resource::<EditorStateResource>()
        .read_resource::<InputResource>()
        .read_resource::<ViewportResource>()
        .write_resource::<EditorSelectionResource>()
        .write_resource::<DebugDraw3DResource>()
        .write_resource::<DebugDraw2DResource>()
        .write_resource::<EditorDraw3DResource>()
        .read_resource::<UniverseResource>()
        .build(
            |_command_buffer,
             _subworld,
             (
                _editor_state,
                input_state,
                viewport,
                _editor_selection,
                debug_draw_3d,
                debug_draw_2d,
                editor_draw,
                _universe_resource,
            ),
            _| {
                editor_draw.update(input_state.input_state(), &*viewport, debug_draw_3d, debug_draw_2d);
            },
        )
}
