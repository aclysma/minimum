use legion::prelude::*;

use crate::resources::{
    EditorStateResource, InputResource, TimeResource, EditorSelectionResource, ViewportResource,
    DebugDrawResource, UniverseResource, EditorDrawResource, EditorTransaction, CameraResource,
};
use crate::resources::ImguiResource;
use crate::resources::EditorTool;
use legion_transaction::{TransactionBuilder, Transaction};

use imgui;
use skulpin::app::VirtualKeyCode;
use skulpin::LogicalSize;
use skulpin::app::MouseScrollDelta;
use skulpin::app::MouseButton;
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

mod main_menu;
pub use main_menu::editor_imgui_menu;

mod entity_list_window;
pub use entity_list_window::editor_entity_list_window;

mod inspector_window;
pub use inspector_window::editor_inspector_window;

mod selection;
pub use selection::draw_selection_shapes;
pub use selection::editor_handle_selection;

mod gizmos;
pub use gizmos::editor_gizmos;

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
                if input_state.is_key_just_down(VirtualKeyCode::Key1) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Translate);
                }

                if input_state.is_key_just_down(VirtualKeyCode::Key2) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Scale);
                }

                if input_state.is_key_just_down(VirtualKeyCode::Key3) {
                    editor_state.enqueue_set_active_editor_tool(EditorTool::Rotate);
                }

                if input_state.is_key_just_down(VirtualKeyCode::Space) {
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
            |command_buffer, subworld, (input_state, camera_resource, viewport_resource), _| {
                // Right click drag pans the viewport
                if let Some(mouse_drag) = input_state.mouse_drag_in_progress(MouseButton::Right) {
                    let mut delta = mouse_drag.world_scale_previous_frame_delta;
                    delta *= glam::Vec2::new(-1.0, -1.0);
                    camera_resource.position += delta;
                }

                // Right click drag pans the viewport
                let mouse_scroll = input_state.mouse_wheel_delta();
                let mut delta = match mouse_scroll {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
                };

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
                editor_draw.update(input_state.input_state(), &*viewport);
            },
        )
}
