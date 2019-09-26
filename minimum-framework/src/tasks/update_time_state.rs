use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::resources::InputState;
use crate::resources::FrameworkOptions;
use crate::resources::{FrameworkActionQueue, TimeState};

pub struct UpdateTimeState;
pub type UpdateTimeStateTask = crate::base::ResourceTask<UpdateTimeState>;
impl ResourceTaskImpl for UpdateTimeState {
    type RequiredResources = (
        Write<TimeState>,
        Read<InputState>,
        Write<FrameworkActionQueue>,
        Read<FrameworkOptions>
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseFrameBegin>();
    }

    fn run(
        context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use crate::PlayMode;
        let (mut time_state, input_state, mut game_control, framework_options) = data;

        let play_mode = if context_flags.flags() & crate::context_flags::PLAYMODE_PLAYING != 0 {
            PlayMode::Playing
        } else if context_flags.flags() & crate::context_flags::PLAYMODE_PAUSED != 0 {
            PlayMode::Paused
        } else {
            PlayMode::System
        };

        time_state.update(play_mode);

        if input_state.is_key_just_down(framework_options.keybinds.edit_play_toggle) {
            let new_play_mode = match play_mode {
                PlayMode::System => PlayMode::Playing,
                PlayMode::Paused => PlayMode::Playing,
                PlayMode::Playing => PlayMode::System,
            };

            game_control.enqueue_change_play_mode(new_play_mode);
        }
    }
}
