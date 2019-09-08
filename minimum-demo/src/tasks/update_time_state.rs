use rendy::wsi::winit;

use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ResourceTaskImpl, TaskConfig};

use framework::resources::{TimeState, FrameworkActionQueue};
use crate::resources::{InputManager};
use named_type::NamedType;

#[derive(NamedType)]
pub struct UpdateTimeState;
pub type UpdateTimeStateTask = minimum::ResourceTask<UpdateTimeState>;
impl ResourceTaskImpl for UpdateTimeState {
    type RequiredResources = (Write<TimeState>, Read<InputManager>, Write<FrameworkActionQueue>);
    //const REQUIRED_FLAGS: usize = 0;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseFrameBegin>();
    }

    fn run(
        //&mut self,
        //task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use framework::PlayMode;
        let (mut time_state, input_manager, mut game_control) = data;

        //TODO: Replace this code
//        let play_mode =
//            if task_context.context_flags() & framework::context_flags::PLAYMODE_PLAYING != 0 {
//                PlayMode::Playing
//            } else if task_context.context_flags() & framework::context_flags::PLAYMODE_PAUSED != 0 {
//                PlayMode::Paused
//            } else {
//                PlayMode::System
//            };

        let play_mode = PlayMode::Playing;
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
