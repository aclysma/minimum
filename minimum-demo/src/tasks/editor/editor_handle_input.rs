use minimum::resource::{DataRequirement, Read, Write};
use minimum::ComponentStorage;
use minimum::{Task, TaskContext, WriteComponent};

use crate::resources::{DebugDraw, EditorCollisionWorld, InputManager, MouseButtons, RenderState};

use crate::components::EditorSelectedComponent;
use ncollide2d::world::CollisionGroups;

use winit::event::VirtualKeyCode;
use named_type::NamedType;

#[derive(NamedType)]
pub struct EditorHandleInput;
impl Task for EditorHandleInput {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<RenderState>,
        Read<EditorCollisionWorld>,
        WriteComponent<EditorSelectedComponent>,
        Write<DebugDraw>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_SYSTEM;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            _entity_set,
            input_manager,
            render_state,
            editor_collision_world,
            mut editor_selected_components,
            mut debug_draw,
        ) = data;

        if task_context.context_flags()
            & (crate::context_flags::PLAYMODE_PAUSED | crate::context_flags::PLAYMODE_PLAYING)
            != 0
        {
            return;
        }

        // Escape cancels the selection
        if input_manager.is_key_just_down(VirtualKeyCode::Escape) {
            editor_selected_components.free_all();
        }

        // This will contain the entities to operate on, or None if we haven't issues a select operation
        let mut new_selection: Option<Vec<_>> = None;

        let selection_collision_group = CollisionGroups::new();

        if let Some(drag_complete) = input_manager.mouse_drag_just_finished(MouseButtons::Left) {
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
        } else if let Some(clicked) =
            input_manager.mouse_button_just_clicked_position(MouseButtons::Left)
        {
            // Clicked, do a raycast
            let target_position = render_state.ui_space_to_world_space(clicked).into();

            let results = editor_collision_world
                .world()
                .interferences_with_point(&target_position, &selection_collision_group);

            new_selection = Some(results.map(|x| x.data()).collect());
        } else if let Some(drag_in_progress) =
            input_manager.mouse_drag_in_progress(MouseButtons::Left)
        {
            // Dragging, draw a rectangle
            debug_draw.add_rect(
                render_state.ui_space_to_world_space(drag_in_progress.begin_position),
                render_state.ui_space_to_world_space(drag_in_progress.end_position),
                glm::vec4(1.0, 1.0, 0.0, 1.0),
            );
        }

        if let Some(entities) = new_selection {

            let add_to_selection = input_manager.is_key_down(VirtualKeyCode::LShift) || input_manager.is_key_down(VirtualKeyCode::RShift);
            let subtract_from_selection = input_manager.is_key_down(VirtualKeyCode::LAlt) || input_manager.is_key_down(VirtualKeyCode::RAlt);

            // default selecting behavior is to drop the old selection
            if !add_to_selection && !subtract_from_selection {
                editor_selected_components.free_all();
            }

            for entity in entities {
                if subtract_from_selection {
                    editor_selected_components.free_if_exists(entity);
                } else {
                    if !editor_selected_components.exists(entity) {
                        editor_selected_components.allocate(entity, EditorSelectedComponent::new());
                    }
                }
            }
        }
    }
}
