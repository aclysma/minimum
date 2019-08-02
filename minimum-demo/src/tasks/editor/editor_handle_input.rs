use minimum::systems::{DataRequirement, Read, Write};
use minimum::ComponentStorage;
use minimum::{EntityHandle, Task, TaskContext, WriteComponent};

use crate::resources::{DebugDraw, EditorCollisionWorld, InputManager, MouseButtons, RenderState};

use crate::components::EditorSelectedComponent;
use crate::tasks::DebugDrawComponents;
use ncollide2d::world::CollisionGroups;

#[derive(typename::TypeName)]
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

        if let Some(drag_complete) = input_manager.mouse_drag_just_finished(MouseButtons::Left) {
            println!("just dragged {:?}", drag_complete);
        } else if let Some(drag_in_progress) =
            input_manager.mouse_drag_in_progress(MouseButtons::Left)
        {
            debug_draw.add_rect(
                render_state.ui_space_to_world_space(drag_in_progress.begin_position),
                render_state.ui_space_to_world_space(drag_in_progress.end_position),
                glm::vec4(1.0, 1.0, 0.0, 1.0),
            );
        } else if let Some(clicked) =
            input_manager.mouse_button_just_clicked_position(MouseButtons::Left)
        {
            let target_position = render_state.ui_space_to_world_space(clicked).into();
            let collision_group = CollisionGroups::new();

            println!("click {:?}", target_position);

            let world: &ncollide2d::world::CollisionWorld<f32, EntityHandle> =
                editor_collision_world.world();
            let results = world.interferences_with_point(&target_position, &collision_group);

            editor_selected_components.free_all();

            for result in results {
                println!("found a thing");
                //TODO: Do selection logic
                editor_selected_components.allocate(result.data(), EditorSelectedComponent::new());
            }
        }
    }
}
