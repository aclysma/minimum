use minimum::systems::{DataRequirement, Read, Write};
use minimum::{DispatchControl, Task, TaskContext};

use crate::resources::{InputManager, TimeState};

#[derive(typename::TypeName)]
pub struct UpdateTimeState;
impl Task for UpdateTimeState {
    type RequiredResources = (Write<TimeState>, Read<InputManager>, Write<DispatchControl>);
    const REQUIRED_FLAGS: usize = 0;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use crate::PlayMode;
        let (mut time_state, input_manager, mut dispatch_control) = data;

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

            // Clear playmode flags
            *dispatch_control.next_frame_context_flags_mut() &=
                !(crate::context_flags::PLAYMODE_SYSTEM
                    | crate::context_flags::PLAYMODE_PAUSED
                    | crate::context_flags::PLAYMODE_PLAYING);

            // Set the appropriate ones
            match new_play_mode {
                PlayMode::System => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM
                }
                PlayMode::Paused => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM
                            | crate::context_flags::PLAYMODE_PAUSED
                }
                PlayMode::Playing => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM
                            | crate::context_flags::PLAYMODE_PAUSED
                            | crate::context_flags::PLAYMODE_PLAYING
                }
            }
        }
    }
}
