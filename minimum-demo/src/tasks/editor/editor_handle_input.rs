use minimum::systems::{DataRequirement, Read};
use minimum::{Task, TaskContext};

use crate::resources::{EditorCollisionWorld, InputManager, MouseButtons, RenderState};

use ncollide2d::world::CollisionGroups;

#[derive(typename::TypeName)]
pub struct EditorHandleInput;
impl Task for EditorHandleInput {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<RenderState>,
        Read<EditorCollisionWorld>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_SYSTEM;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (_entity_set, input_manager, render_state, editor_collision_world) = data;

        if task_context.context_flags()
            & (crate::context_flags::PLAYMODE_PAUSED | crate::context_flags::PLAYMODE_PLAYING)
            != 0
        {
            return;
        }

        if input_manager.is_mouse_just_down(MouseButtons::Left) {
            let target_position = render_state
                .ui_space_to_world_space(input_manager.mouse_position())
                .into();
            let collision_group = CollisionGroups::new();

            println!("click {:?}", target_position);

            let world: &ncollide2d::world::CollisionWorld<f32, ()> = editor_collision_world.world();
            let results = world.interferences_with_point(&target_position, &collision_group);

            for _result in results {
                println!("found a thing");
                //TODO: Do selection logic
            }
        }
    }
}
