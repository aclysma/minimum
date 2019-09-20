use minimum::resource::{DataRequirement, Read, Write};
use minimum::ComponentStorage;
use minimum::{ResourceTaskImpl, TaskConfig, TaskContextFlags, WriteComponent, Component, EntitySet};

use crate::resources::{DebugDraw, InputManager, MouseButtons, RenderState, EditorDraw};
use framework::resources::editor::{EditorCollisionWorld, EditorTool, EditorUiState};
use framework::components::PersistentEntityComponent;

use framework::components::editor::EditorSelectedComponent;
use framework::components::editor::EditorModifiedComponent;

use ncollide2d::world::CollisionGroups;
use crate::components::TransformComponent;
use crate::components::TransformComponentPrototype;

use rendy::wsi::winit;
use winit::event::VirtualKeyCode;

pub struct EditorHandleInput;
pub type EditorHandleInputTask = minimum::ResourceTask<EditorHandleInput>;
impl ResourceTaskImpl for EditorHandleInput {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<RenderState>,
        Read<EditorCollisionWorld>,
        WriteComponent<EditorSelectedComponent>,
        Write<DebugDraw>,
        Write<EditorUiState>,
        Write<EditorDraw>,
        WriteComponent<TransformComponent>,
        WriteComponent<PersistentEntityComponent>,
        WriteComponent<EditorModifiedComponent>
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.this_uses_data_from::<crate::tasks::editor::EditorUpdateSelectionWorldTask>();
        config.run_only_if(framework::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            input_manager,
            render_state,
            editor_collision_world,
            mut editor_selected_components,
            mut debug_draw,
            mut editor_ui_state,
            mut editor_draw,
            mut transform_components,
            mut persistent_entity_components,
            mut editor_modified_components
        ) = data;

        if input_manager.is_key_just_down(VirtualKeyCode::Key1) {
            editor_ui_state.active_editor_tool = EditorTool::Translate;
        }

        if input_manager.is_key_just_down(VirtualKeyCode::Key2) {
            editor_ui_state.active_editor_tool = EditorTool::Scale;
        }

        if input_manager.is_key_just_down(VirtualKeyCode::Key3) {
            editor_ui_state.active_editor_tool = EditorTool::Rotate;
        }

        if context_flags.flags()
            & (framework::context_flags::PLAYMODE_PAUSED
                | framework::context_flags::PLAYMODE_PLAYING)
            != 0
        {
            return;
        }

        // Escape cancels the selection
        if input_manager.is_key_just_down(VirtualKeyCode::Escape) {
            editor_selected_components.free_all();
        }

        editor_draw.update(&*input_manager, &*render_state);


        handle_translate_gizmo_input(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, &mut *transform_components, &mut *persistent_entity_components, &mut *editor_modified_components);
        handle_scale_gizmo_input(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, &mut *transform_components, &mut *persistent_entity_components, &mut *editor_modified_components);
        handle_select_input(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, & *transform_components);

        match editor_ui_state.active_editor_tool {
            //EditorTool::Select => handle_select_tool_input(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state),
            EditorTool::Translate => draw_translate_gizmo(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, &* transform_components),
            EditorTool::Scale => draw_scale_gizmo(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, &* transform_components),
            EditorTool::Rotate => draw_rotate_gizmo(&*entity_set, &*input_manager, &* render_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state, &mut *editor_draw, &* transform_components)
        }
    }
}

fn handle_translate_gizmo_input(
    entity_set: &EntitySet,
    _input_manager: &InputManager,
    _render_state: &RenderState,
    _editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    _debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    transform_components: &mut <TransformComponent as Component>::Storage,
    persistent_entity_components: &mut <PersistentEntityComponent as Component>::Storage,
    editor_modified_components: &mut <EditorModifiedComponent as Component>::Storage

) {
    if let Some(drag_in_progress) = editor_draw.shape_drag_in_progress_or_just_finished(MouseButtons::Left) {
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
            return;
        }

        // Determine the drag distance in ui_space
        let mut world_space_previous_frame_delta = drag_in_progress.world_space_previous_frame_delta;
        let mut world_space_accumulated_delta = drag_in_progress.world_space_accumulated_frame_delta;
        if !translate_x {
            world_space_previous_frame_delta.x = 0.0;
            world_space_accumulated_delta.x = 0.0;
        }

        if !translate_y {
            world_space_previous_frame_delta.y = 0.0;
            world_space_accumulated_delta.y = 0.0;
        }

        for (entity_handle, _) in editor_selected_components.iter(&entity_set) {

            // If we are ending the drag and manage to find a persistent component prototype, we will
            // update that and recreate the object. In which case, we can skip updating the transform
            // component itself.
            let mut update_transform_component = true;
            if editor_draw.is_shape_drag_just_finished(MouseButtons::Left) {
                // update the prototype and invalidate the object
                if let Some(persistent_entity_component) = persistent_entity_components.get_mut(&entity_handle) {
                    let mut entity_prototype = persistent_entity_component.entity_prototype_mut().lock();
                    if let Some(transform_component_prototype) = entity_prototype.find_component_prototype_mut::<TransformComponentPrototype>() {

                        // Edit the prototype
                        *transform_component_prototype.data_mut().position_mut() += world_space_accumulated_delta;

                        // Mark the object as needing to be recreated
                        if !editor_modified_components.exists(&entity_handle) {
                            editor_modified_components.allocate(&entity_handle, EditorModifiedComponent::new()).unwrap();
                        }

                        // Skip updating the transform component
                        update_transform_component = false;
                    }
                }
            }

            if update_transform_component {
                if let Some(transform_component) = transform_components.get_mut(&entity_handle) {
                    // Edit the component - recompute a new position. This is done using original values
                    // to avoid fp innacuracy. This is just a preview. We don't commit the change until the
                    // drag is complete.
                    *transform_component.position_mut() += world_space_previous_frame_delta;
                    transform_component.requires_sync_to_physics();
                }
            }
        }
    }
}

fn draw_translate_gizmo(
    entity_set: &EntitySet,
    _input_manager: &InputManager,
    _render_state: &RenderState,
    _editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    transform_components: &<TransformComponent as Component>::Storage
) {
    for (entity, _) in editor_selected_components.iter(&entity_set) {
        if let Some(transform) = transform_components.get(&entity) {
            let position = transform.position();

            //TODO: Make this resolution independent. Need a UI multiplier?
            editor_draw.add_line(
                "x_axis_translate",
                debug_draw,
                position,
                position + glm::vec2(100.0, 0.0),
                glm::vec4(0.0, 0.0, 1.0, 1.0)
            );

            editor_draw.add_line(
                "y_axis_translate",
                debug_draw,
                position,
                position + glm::vec2(0.0, 100.0),
                glm::vec4(0.0, 1.0, 0.0, 1.0)
            );

            editor_draw.add_line(
                "xy_axis_translate",
                debug_draw,
                position + glm::vec2(0.0, 25.0),
                position + glm::vec2(25.0, 25.0),
                glm::vec4(1.0, 1.0, 0.0, 1.0)
            );

            editor_draw.add_line(
                "xy_axis_translate",
                debug_draw,
                position + glm::vec2(25.0, 0.0),
                position + glm::vec2(25.0, 25.0),
                glm::vec4(1.0, 1.0, 0.0, 1.0)
            );
        }
    }
}


fn handle_scale_gizmo_input(
    entity_set: &EntitySet,
    _input_manager: &InputManager,
    _render_state: &RenderState,
    _editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    _debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    transform_components: &mut <TransformComponent as Component>::Storage,
    persistent_entity_components: &mut <PersistentEntityComponent as Component>::Storage,
    editor_modified_components: &mut <EditorModifiedComponent as Component>::Storage

) {
    if let Some(drag_in_progress) = editor_draw.shape_drag_in_progress_or_just_finished(MouseButtons::Left) {
        // See what if any axis we will operate on
        let mut translate_x = false;
        let mut translate_y = false;
        if drag_in_progress.shape_id == "x_axis_scale" {
            translate_x = true;
        } else if drag_in_progress.shape_id == "y_axis_scale" {
            translate_y = true;
        } else if drag_in_progress.shape_id == "xy_axis_scale" {
            translate_x = true;
            translate_y = true;
        }

        // Early out if we didn't touch either axis
        if !translate_x && !translate_y {
            return;
        }

        // Determine the drag distance in ui_space
        //TODO: I was intending this to use ui space but the values during drag are not lining up
        // with values on end drag. This is likely an fp precision issue.
        let mut ui_space_previous_frame_delta = drag_in_progress.world_space_previous_frame_delta;
        let mut ui_space_accumulated_delta = drag_in_progress.world_space_accumulated_frame_delta;
        if !translate_x {
            ui_space_previous_frame_delta.x = 0.0;
            ui_space_accumulated_delta.x = 0.0;
        }

        if !translate_y {
            ui_space_previous_frame_delta.y = 0.0;
            ui_space_accumulated_delta.y = 0.0;
        }

        for (entity_handle, _) in editor_selected_components.iter(&entity_set) {

            // If we are ending the drag and manage to find a persistent component prototype, we will
            // update that and recreate the object. In which case, we can skip updating the transform
            // component itself.
            let mut update_transform_component = true;
            if editor_draw.is_shape_drag_just_finished(MouseButtons::Left) {
                // update the prototype and invalidate the object
                if let Some(persistent_entity_component) = persistent_entity_components.get_mut(&entity_handle) {
                    let mut entity_prototype = persistent_entity_component.entity_prototype_mut().lock();
                    if let Some(transform_component_prototype) = entity_prototype.find_component_prototype_mut::<TransformComponentPrototype>() {

                        // Edit the prototype
                        *transform_component_prototype.data_mut().scale_mut() += ui_space_accumulated_delta * 0.1;

                        // Mark the object as needing to be recreated
                        if !editor_modified_components.exists(&entity_handle) {
                            editor_modified_components.allocate(&entity_handle, EditorModifiedComponent::new()).unwrap();
                        }

                        // Skip updating the transform component
                        update_transform_component = false;
                    }
                }
            }

            if update_transform_component {
                if let Some(transform_component) = transform_components.get_mut(&entity_handle) {
                    // Edit the component - recompute a new position. This is done using original values
                    // to avoid fp innacuracy. This is just a preview. We don't commit the change until the
                    // drag is complete.
                    *transform_component.scale_mut() += ui_space_previous_frame_delta * 0.1;
                    transform_component.requires_sync_to_physics();
                }
            }
        }
    }
}

fn draw_scale_gizmo(
    entity_set: &EntitySet,
    _input_manager: &InputManager,
    _render_state: &RenderState,
    _editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    transform_components: &<TransformComponent as Component>::Storage
) {
    for (entity, _) in editor_selected_components.iter(&entity_set) {
        if let Some(transform) = transform_components.get(&entity) {
            let position = transform.position();

            //TODO: Make this resolution independent. Need a UI multiplier?
            editor_draw.add_line(
                "x_axis_scale",
                debug_draw,
                position,
                position + glm::vec2(100.0, 0.0),
                glm::vec4(0.0, 0.0, 1.0, 1.0)
            );

            editor_draw.add_line(
                "y_axis_scale",
                debug_draw,
                position,
                position + glm::vec2(0.0, 100.0),
                glm::vec4(0.0, 1.0, 0.0, 1.0)
            );

            editor_draw.add_line(
                "xy_axis_scale",
                debug_draw,
                position + glm::vec2(0.0, 25.0),
                position + glm::vec2(25.0, 25.0),
                glm::vec4(1.0, 1.0, 0.0, 1.0)
            );

            editor_draw.add_line(
                "xy_axis_scale",
                debug_draw,
                position + glm::vec2(25.0, 0.0),
                position + glm::vec2(25.0, 25.0),
                glm::vec4(1.0, 1.0, 0.0, 1.0)
            );
        }
    }
}

fn draw_rotate_gizmo(
    entity_set: &EntitySet,
    _input_manager: &InputManager,
    _render_state: &RenderState,
    _editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    transform_components: &<TransformComponent as Component>::Storage
) {
    for (entity, _) in editor_selected_components.iter(&entity_set) {
        if let Some(pos) = transform_components.get(&entity) {
            let position = pos.position();

            //TODO: Make this resolution independent. Need a UI multiplier?
            editor_draw.add_circle_outline(
                "z_axis_rotate",
                debug_draw,
                position,
                150.0,
                glm::vec4(0.0, 1.0, 0.0, 1.0)
            );
        }
    }
}

fn handle_select_input(
    _entity_set: &EntitySet,
    input_manager: &InputManager,
    render_state: &RenderState,
    editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    _editor_ui_state: &EditorUiState,
    editor_draw: &mut EditorDraw,
    _transform_components: &<TransformComponent as Component>::Storage
) {
    // This will contain the entities to operate on, or None if we haven't issues a select operation
    let mut new_selection: Option<Vec<_>> = None;

    let selection_collision_group = CollisionGroups::new();

    if editor_draw.is_interacting_with_anything() {
        // drop input, the clicking/dragging is happening on editor shapes
    } else if let Some(drag_complete) = input_manager.mouse_drag_just_finished(MouseButtons::Left) {
        // Drag complete, check AABB
        let target_position0: glm::Vec2 = render_state
            .ui_space_to_world_space(drag_complete.begin_position)
            .into();
        let target_position1: glm::Vec2 = render_state
            .ui_space_to_world_space(drag_complete.end_position)
            .into();

        let mins = glm::vec2(
            f32::min(target_position0.x, target_position1.x),
            f32::min(target_position0.y, target_position1.y),
        );

        let maxs = glm::vec2(
            f32::max(target_position0.x, target_position1.x),
            f32::max(target_position0.y, target_position1.y),
        );

        let aabb = ncollide2d::bounding_volume::AABB::new(
            nalgebra::Point::from(mins),
            nalgebra::Point::from(maxs),
        );

        let results = editor_collision_world
            .world()
            .interferences_with_aabb(&aabb, &selection_collision_group);

        new_selection = Some(results.map(|x| x.data()).collect());
    } else if let Some(clicked) = input_manager.mouse_button_just_clicked_position(MouseButtons::Left) {
        // Clicked, do a raycast
        let target_position = render_state.ui_space_to_world_space(clicked).into();

        let results = editor_collision_world
            .world()
            .interferences_with_point(&target_position, &selection_collision_group);

        new_selection = Some(results.map(|x| x.data()).collect());
    } else if let Some(drag_in_progress) = input_manager.mouse_drag_in_progress(MouseButtons::Left) {
        // Dragging, draw a rectangle
        debug_draw.add_rect(
            render_state.ui_space_to_world_space(drag_in_progress.begin_position),
            render_state.ui_space_to_world_space(drag_in_progress.end_position),
            glm::vec4(1.0, 1.0, 0.0, 1.0),
        );
    }

    if let Some(entities) = new_selection {
        let add_to_selection = input_manager.is_key_down(VirtualKeyCode::LShift)
            || input_manager.is_key_down(VirtualKeyCode::RShift);
        let subtract_from_selection = input_manager.is_key_down(VirtualKeyCode::LAlt)
            || input_manager.is_key_down(VirtualKeyCode::RAlt);

        // default selecting behavior is to drop the old selection
        if !add_to_selection && !subtract_from_selection {
            editor_selected_components.free_all();
        }

        for entity in entities {
            if subtract_from_selection {
                editor_selected_components.free_if_exists(entity);
            } else {
                if !editor_selected_components.exists(entity) {
                    editor_selected_components.allocate(entity, EditorSelectedComponent::new()).unwrap();
                }
            }
        }
    }
}