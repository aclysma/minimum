use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::ComponentStorage;
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags, WriteComponent, Component, EntitySet};

use crate::framework::resources::DebugDraw;
use crate::framework::resources::InputState;
use crate::framework::resources::MouseButton;
use crate::framework::resources::FrameworkOptions;
use crate::framework::resources::editor::EditorDraw;
use crate::framework::resources::CameraState;
use crate::framework::resources::editor::{EditorCollisionWorld, EditorTool, EditorUiState};
use crate::framework::components::PersistentEntityComponent;

use crate::framework::components::editor::EditorSelectedComponent;
use crate::framework::components::editor::EditorModifiedComponent;

use ncollide::world::CollisionGroups;
use crate::framework::components::TransformComponent;
use crate::framework::components::TransformComponentPrototype;

pub struct EditorHandleInput;
pub type EditorHandleInputTask = crate::base::ResourceTask<EditorHandleInput>;
impl ResourceTaskImpl for EditorHandleInput {
    type RequiredResources = (
        Read<crate::base::EntitySet>,
        Read<InputState>,
        Read<CameraState>,
        Read<EditorCollisionWorld>,
        WriteComponent<EditorSelectedComponent>,
        Write<DebugDraw>,
        Write<EditorUiState>,
        Write<EditorDraw>,
        WriteComponent<TransformComponent>,
        WriteComponent<PersistentEntityComponent>,
        WriteComponent<EditorModifiedComponent>,
        Read<FrameworkOptions>
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePreRender>();
        config.this_uses_data_from::<crate::tasks::editor::EditorUpdateSelectionWorldTask>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            input_state,
            camera_state,
            editor_collision_world,
            mut editor_selected_components,
            mut debug_draw,
            mut editor_ui_state,
            mut editor_draw,
            mut transform_components,
            mut persistent_entity_components,
            mut editor_modified_components,
            framework_options
        ) = data;

        if input_state.is_key_just_down(framework_options.keybinds.translate_tool) {
            editor_ui_state.active_editor_tool = EditorTool::Translate;
        }

        if input_state.is_key_just_down(framework_options.keybinds.scale_tool) {
            editor_ui_state.active_editor_tool = EditorTool::Scale;
        }

        if input_state.is_key_just_down(framework_options.keybinds.rotate_tool) {
            editor_ui_state.active_editor_tool = EditorTool::Rotate;
        }

        if context_flags.flags()
            & (crate::framework::context_flags::PLAYMODE_PAUSED
                | crate::framework::context_flags::PLAYMODE_PLAYING)
            != 0
        {
            return;
        }

        // Escape cancels the selection
        if input_state.is_key_just_down(framework_options.keybinds.clear_selection) {
            editor_selected_components.free_all();
        }

        editor_draw.update(&*input_state, &*camera_state);


        handle_translate_gizmo_input(&*entity_set,  &mut* editor_selected_components,  &mut *editor_draw, &mut *transform_components, &mut *persistent_entity_components, &mut *editor_modified_components);
        handle_scale_gizmo_input(&*entity_set, &mut* editor_selected_components, &mut *editor_draw, &mut *transform_components, &mut *persistent_entity_components, &mut *editor_modified_components);
        handle_rotate_gizmo_input(&*entity_set,  &mut* editor_selected_components, &mut *editor_draw, &mut *transform_components, &mut *persistent_entity_components, &mut *editor_modified_components);

        handle_select_input( &*input_state, &* camera_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &mut *editor_draw, &* framework_options);

        match editor_ui_state.active_editor_tool {
            //EditorTool::Select => handle_select_tool_input(&*entity_set, &*input_state, &* camera_state, &* editor_collision_world, &mut* editor_selected_components, &mut*debug_draw, &editor_ui_state),
            EditorTool::Translate => draw_translate_gizmo(&*entity_set, &mut* editor_selected_components, &mut*debug_draw, &mut *editor_draw, &* transform_components),
            EditorTool::Scale => draw_scale_gizmo(&*entity_set, &mut* editor_selected_components, &mut*debug_draw, &mut *editor_draw, &* transform_components),
            EditorTool::Rotate => draw_rotate_gizmo(&*entity_set, &mut* editor_selected_components, &mut*debug_draw, &mut *editor_draw, &* transform_components)
        }
    }
}

fn handle_translate_gizmo_input(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    editor_draw: &mut EditorDraw,
    transform_components: &mut <TransformComponent as Component>::Storage,
    persistent_entity_components: &mut <PersistentEntityComponent as Component>::Storage,
    editor_modified_components: &mut <EditorModifiedComponent as Component>::Storage

) {
    if let Some(drag_in_progress) = editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left) {
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
            if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
                // update the prototype and invalidate the object
                if let Some(persistent_entity_component) = persistent_entity_components.get_mut(&entity_handle) {
                    let mut entity_prototype = persistent_entity_component.entity_prototype_mut().lock();
                    if let Some(transform_component_prototype) = entity_prototype.find_component_prototype_mut::<TransformComponentPrototype>() {

                        // Edit the prototype
                        #[cfg(feature = "dim3")]
                        let world_space_accumulated_delta = glm::vec2_to_vec3(&world_space_accumulated_delta);
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
                    #[cfg(feature = "dim3")]
                    let world_space_previous_frame_delta = glm::vec2_to_vec3(&world_space_previous_frame_delta);
                    *transform_component.position_mut() += world_space_previous_frame_delta;
                    transform_component.requires_sync_to_physics();
                }
            }
        }
    }
}

fn draw_translate_gizmo(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    editor_draw: &mut EditorDraw,
    transform_components: &<TransformComponent as Component>::Storage
) {
    for (entity, _) in editor_selected_components.iter(&entity_set) {
        if let Some(transform) = transform_components.get(&entity) {
            let position = transform.position();

            let x_color = glm::vec4(0.0, 1.0, 0.0, 1.0);
            let y_color = glm::vec4(1.0, 0.6, 0.0, 1.0);
            let xy_color = glm::vec4(1.0, 1.0, 0.0, 1.0);

            //TODO: Make this resolution independent. Need a UI multiplier?

            // x axis line
            editor_draw.add_line(
                "x_axis_translate",
                debug_draw,
                position.xy(),
                position.xy() + glm::vec2(100.0, 0.0),
                x_color
            );

            editor_draw.add_line(
                "x_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(85.0, 15.0),
                position.xy() + glm::vec2(100.0, 0.0),
                x_color
            );

            editor_draw.add_line(
                "x_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(85.0, -15.0),
                position.xy() + glm::vec2(100.0, 0.0),
                x_color
            );

            // y axis line
            editor_draw.add_line(
                "y_axis_translate",
                debug_draw,
                position.xy(),
                position.xy() + glm::vec2(0.0, 100.0),
                y_color
            );

            editor_draw.add_line(
                "y_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(-15.0, 85.0),
                position.xy() + glm::vec2(0.0, 100.0),
                y_color
            );

            editor_draw.add_line(
                "y_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(15.0, 85.0),
                position.xy() + glm::vec2(0.0, 100.0),
                y_color
            );

            // xy line
            editor_draw.add_line(
                "xy_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(0.0, 25.0),
                position.xy() + glm::vec2(25.0, 25.0),
                xy_color
            );

            // xy line
            editor_draw.add_line(
                "xy_axis_translate",
                debug_draw,
                position.xy() + glm::vec2(25.0, 0.0),
                position.xy() + glm::vec2(25.0, 25.0),
                xy_color
            );
        }
    }
}

fn sign_aware_magnitude(v: glm::Vec2) -> f32 {
    let mut total = 0.0;
    total += if v.x > 0.0 {
        v.x * v.x
    } else {
        v.x * v.x * -1.0
    };

    total += if v.y > 0.0 {
        v.y * v.y
    } else {
        v.y * v.y * -1.0
    };

    if total >= 0.0 {
        total.sqrt()
    } else {
        (total * -1.0).sqrt() * -1.0
    }
}

fn handle_scale_gizmo_input(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    editor_draw: &mut EditorDraw,
    transform_components: &mut <TransformComponent as Component>::Storage,
    persistent_entity_components: &mut <PersistentEntityComponent as Component>::Storage,
    editor_modified_components: &mut <EditorModifiedComponent as Component>::Storage

) {
    if let Some(drag_in_progress) = editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left) {
        // See what if any axis we will operate on
        let mut translate_x = false;
        let mut translate_y = false;
        let mut uniform_scale = false;
        if drag_in_progress.shape_id == "x_axis_scale" {
            translate_x = true;
        } else if drag_in_progress.shape_id == "y_axis_scale" {
            translate_y = true;
        } else if drag_in_progress.shape_id == "uniform_scale" {
            uniform_scale = true;
        }

        // Early out if we didn't touch either axis
        if !translate_x && !translate_y && !uniform_scale {
            return;
        }

        // Determine the drag distance in ui_space
        //TODO: I was intending this to use ui space but the values during drag are not lining up
        // with values on end drag. This is likely an fp precision issue.
        let mut ui_space_previous_frame_delta = drag_in_progress.world_space_previous_frame_delta;
        let mut ui_space_accumulated_delta = drag_in_progress.world_space_accumulated_frame_delta;
        if !translate_x && !uniform_scale {
            ui_space_previous_frame_delta.x = 0.0;
            ui_space_accumulated_delta.x = 0.0;
        }

        if !translate_y && !uniform_scale {
            ui_space_previous_frame_delta.y = 0.0;
            ui_space_accumulated_delta.y = 0.0;
        }

        if uniform_scale {
            ui_space_previous_frame_delta.x = sign_aware_magnitude(ui_space_previous_frame_delta);
            ui_space_previous_frame_delta.y = ui_space_previous_frame_delta.x;
            ui_space_accumulated_delta.x = sign_aware_magnitude(ui_space_accumulated_delta);
            ui_space_accumulated_delta.y = ui_space_accumulated_delta.x;
        }

        for (entity_handle, _) in editor_selected_components.iter(&entity_set) {

            // If we are ending the drag and manage to find a persistent component prototype, we will
            // update that and recreate the object. In which case, we can skip updating the transform
            // component itself.
            let mut update_transform_component = true;
            if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
                // update the prototype and invalidate the object
                if let Some(persistent_entity_component) = persistent_entity_components.get_mut(&entity_handle) {
                    let mut entity_prototype = persistent_entity_component.entity_prototype_mut().lock();
                    if let Some(transform_component_prototype) = entity_prototype.find_component_prototype_mut::<TransformComponentPrototype>() {

                        // Edit the prototype
                        let adjusted = ui_space_accumulated_delta * 0.05;
                        #[cfg(feature = "dim3")]
                        let adjusted = glm::vec2_to_vec3(&adjusted);
                        *transform_component_prototype.data_mut().scale_mut() += adjusted;

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
                    let adjusted = ui_space_previous_frame_delta * 0.05;
                    #[cfg(feature = "dim3")]
                    let adjusted = glm::vec2_to_vec3(&adjusted);
                    *transform_component.scale_mut() += adjusted;
                    transform_component.requires_sync_to_physics();
                }
            }
        }
    }
}

fn draw_scale_gizmo(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    editor_draw: &mut EditorDraw,
    transform_components: &<TransformComponent as Component>::Storage
) {
    for (entity, _) in editor_selected_components.iter(&entity_set) {
        if let Some(transform) = transform_components.get(&entity) {
            let position = transform.position();

            //TODO: Make this resolution independent. Need a UI multiplier?

            let x_color = glm::vec4(0.0, 1.0, 0.0, 1.0);
            let y_color = glm::vec4(1.0, 0.6, 0.0, 1.0);
            let xy_color = glm::vec4(1.0, 1.0, 0.0, 1.0);

            // x axis line
            editor_draw.add_line(
                "x_axis_scale",
                debug_draw,
                position.xy(),
                position.xy() + glm::vec2(100.0, 0.0),
                x_color
            );

            // x axis line end
            editor_draw.add_line(
                "x_axis_scale",
                debug_draw,
                position.xy() + glm::vec2(100.0, -20.0),
                position.xy() + glm::vec2(100.0, 20.0),
                x_color
            );

            // y axis line
            editor_draw.add_line(
                "y_axis_scale",
                debug_draw,
                position.xy(),
                position.xy() + glm::vec2(0.0, 100.0),
                y_color
            );

            // y axis line end
            editor_draw.add_line(
                "y_axis_scale",
                debug_draw,
                position.xy() + glm::vec2(-20.0, 100.0),
                position.xy() + glm::vec2(20.0, 100.0),
                y_color
            );

            // xy line
            editor_draw.add_line(
                "uniform_scale",
                debug_draw,
                position.xy() + glm::vec2(0.0, 0.0),
                position.xy() + glm::vec2(50.0, 50.0),
                xy_color
            );

            // xy line
            editor_draw.add_line(
                "uniform_scale",
                debug_draw,
                position.xy() + glm::vec2(40.0, 60.0),
                position.xy() + glm::vec2(60.0, 40.0),
                xy_color
            );
        }
    }
}

fn handle_rotate_gizmo_input(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    editor_draw: &mut EditorDraw,
    transform_components: &mut <TransformComponent as Component>::Storage,
    persistent_entity_components: &mut <PersistentEntityComponent as Component>::Storage,
    editor_modified_components: &mut <EditorModifiedComponent as Component>::Storage

) {
    if let Some(drag_in_progress) = editor_draw.shape_drag_in_progress_or_just_finished(MouseButton::Left) {
        // See what if any axis we will operate on
        let mut rotate_z = false;
        if drag_in_progress.shape_id == "z_axis_rotate" {
            rotate_z = true;
        }

        // Early out if we didn't touch either axis
        if !rotate_z {
            return;
        }

        // Determine the drag distance in ui_space
        let ui_space_previous_frame_delta = sign_aware_magnitude(drag_in_progress.world_space_previous_frame_delta);
        let ui_space_accumulated_delta = sign_aware_magnitude(drag_in_progress.world_space_accumulated_frame_delta);

        for (entity_handle, _) in editor_selected_components.iter(&entity_set) {

            // If we are ending the drag and manage to find a persistent component prototype, we will
            // update that and recreate the object. In which case, we can skip updating the transform
            // component itself.
            let mut update_transform_component = true;
            if editor_draw.is_shape_drag_just_finished(MouseButton::Left) {
                // update the prototype and invalidate the object
                if let Some(persistent_entity_component) = persistent_entity_components.get_mut(&entity_handle) {
                    let mut entity_prototype = persistent_entity_component.entity_prototype_mut().lock();
                    if let Some(transform_component_prototype) = entity_prototype.find_component_prototype_mut::<TransformComponentPrototype>() {

                        let adjusted = ui_space_accumulated_delta * 0.05;
                        #[cfg(feature = "dim2")]
                        {
                            // Edit the prototype
                            *transform_component_prototype.data_mut().rotation_mut() += adjusted;
                        }

                        #[cfg(feature = "dim3")]
                        {
                            // Edit the prototype
                            let rotation = glm::quat_angle_axis(adjusted, &glm::Vec3::z());
                            *transform_component_prototype.data_mut().rotation_mut() *= rotation;
                        }

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
                    //*transform_component.scale_mut() += ui_space_previous_frame_delta * 0.05;

                    let adjusted = ui_space_previous_frame_delta * 0.05;
                    #[cfg(feature = "dim2")]
                    {
                        *transform_component.rotation_mut() += adjusted;
                    }

                    #[cfg(feature = "dim3")]
                    {
                        // Edit the prototype
                        let rotation = glm::quat_angle_axis(adjusted, &glm::Vec3::z());
                        *transform_component.rotation_mut() *= rotation;
                    }

                    transform_component.requires_sync_to_physics();
                }
            }
        }
    }
}

fn draw_rotate_gizmo(
    entity_set: &EntitySet,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
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
                position.xy(),
                50.0,
                glm::vec4(0.0, 1.0, 0.0, 1.0)
            );
            editor_draw.add_circle_outline(
                "z_axis_rotate",
                debug_draw,
                position.xy(),
                52.0,
                glm::vec4(0.0, 1.0, 0.0, 1.0)
            );
        }
    }
}

fn handle_select_input(
    input_state: &InputState,
    camera_state: &CameraState,
    editor_collision_world: &EditorCollisionWorld,
    editor_selected_components: &mut <EditorSelectedComponent as Component>::Storage,
    debug_draw: &mut DebugDraw,
    editor_draw: &mut EditorDraw,
    framework_options: &FrameworkOptions
) {
    // This will contain the entities to operate on, or None if we haven't issues a select operation
    let mut new_selection: Option<Vec<_>> = None;

    let selection_collision_group = CollisionGroups::new();

    if editor_draw.is_interacting_with_anything() {
        // drop input, the clicking/dragging is happening on editor shapes
    } else if let Some(drag_complete) = input_state.mouse_drag_just_finished(MouseButton::Left) {
        // Drag complete, check AABB
        let target_position0: glm::Vec2 = camera_state
            .ui_space_to_world_space(drag_complete.begin_position)
            .into();
        let target_position1: glm::Vec2 = camera_state
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

        #[cfg(feature = "dim3")]
        let (mins, maxs) = {
            (glm::vec3(mins.x, mins.y, std::f32::MIN), glm::vec3(maxs.x, maxs.y, std::f32::MAX))
        };

        let aabb = ncollide::bounding_volume::AABB::new(
            nalgebra::Point::from(mins),
            nalgebra::Point::from(maxs),
        );

        let results = editor_collision_world
            .world()
            .interferences_with_aabb(&aabb, &selection_collision_group);

        new_selection = Some(results.map(|x| x.data()).collect());
    } else if let Some(clicked) = input_state.mouse_button_just_clicked_position(MouseButton::Left) {
        // Clicked, do a raycast
        let target_position = camera_state.ui_space_to_world_space(clicked);
        #[cfg(feature = "dim3")]
        let target_position = glm::vec2_to_vec3(&target_position);
        let target_position = ncollide::math::Point::from(target_position);

        let results = editor_collision_world
            .world()
            .interferences_with_point(&target_position, &selection_collision_group);

        new_selection = Some(results.map(|x| x.data()).collect());
    } else if let Some(drag_in_progress) = input_state.mouse_drag_in_progress(MouseButton::Left) {
        // Dragging, draw a rectangle
        debug_draw.add_rect(
            camera_state.ui_space_to_world_space(drag_in_progress.begin_position),
            camera_state.ui_space_to_world_space(drag_in_progress.end_position),
            glm::vec4(1.0, 1.0, 0.0, 1.0),
        );
    }

    if let Some(entities) = new_selection {
        let add_to_selection = input_state.is_key_down(framework_options.keybinds.modify_selection_add1)
            || input_state.is_key_down(framework_options.keybinds.modify_selection_add2);
        let subtract_from_selection = input_state.is_key_down(framework_options.keybinds.modify_selection_subtract1)
            || input_state.is_key_down(framework_options.keybinds.modify_selection_subtract2);

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