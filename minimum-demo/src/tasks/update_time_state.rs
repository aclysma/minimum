use minimum::resource::{DataRequirement, Read, Write};
use minimum::{Task, TaskContext};

use crate::resources::{InputManager, TimeState, FrameworkActionQueue};
use named_type::NamedType;

#[derive(NamedType)]
pub struct UpdateTimeState;
impl Task for UpdateTimeState {
    type RequiredResources = (Write<TimeState>, Read<InputManager>, Write<FrameworkActionQueue>);
    const REQUIRED_FLAGS: usize = 0;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use crate::PlayMode;
        let (mut time_state, input_manager, mut game_control) = data;

        let play_mode =
            if task_context.context_flags() & crate::context_flags::PLAYMODE_PLAYING != 0 {
                PlayMode::Playing
            } else if task_context.context_flags() & crate::context_flags::PLAYMODE_PAUSED != 0 {
                PlayMode::Paused
            } else {
                PlayMode::System
            };

        time_state.update(play_mode);

        use winit::event::VirtualKeyCode;
        if input_manager.is_key_just_down(VirtualKeyCode::Space) {
            let new_play_mode = match play_mode {
                PlayMode::System => PlayMode::Playing,
                PlayMode::Paused => PlayMode::Playing,
                PlayMode::Playing => PlayMode::System,
            };

            game_control.enqueue_change_play_mode(new_play_mode);
        }
    }
}
