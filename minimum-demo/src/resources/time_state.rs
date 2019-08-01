use crate::PlayMode;
use std::time;

const NANOS_PER_SEC: u32 = 1_000_000_000;

//TODO: Exposing duration/instant is a little dangerous, it would be better if durations/instants
// from each mode were different types, and couldn't be used directly with stdlib duration/instants

//TODO: Avoid using pub for fields

// This is not intended to be accessed when the system time updates, but we can double buffer it
// if it becomes a problem
pub struct TimeState {
    // System time that the application started
    pub app_start_system_time: time::SystemTime,

    // rust Instant object captured when the application started
    pub app_start_instant: time::Instant,

    // rust Instant object captured at the start of the frame
    pub previous_instant: time::Instant,

    // The game can be in different levels of play/pause, this determines what mode we are currently in
    pub play_mode: PlayMode,

    play_mode_states: [ModeTimeState; crate::PLAYMODE_COUNT],
}

impl TimeState {
    pub fn new() -> TimeState {
        let now_instant = time::Instant::now();
        let now_system_time = time::SystemTime::now();

        return TimeState {
            app_start_system_time: now_system_time,
            app_start_instant: now_instant,
            previous_instant: now_instant,
            play_mode: PlayMode::Playing,
            play_mode_states: [ModeTimeState::new(); crate::PLAY_MODE_COUNT],
        };
    }

    pub fn update(&mut self, play_mode: PlayMode) {
        // Cache the mode we are in this frame
        self.play_mode = play_mode;

        // Determine length of time since last tick
        let now_instant = time::Instant::now();
        let elapsed = now_instant - self.previous_instant;
        self.previous_instant = now_instant;

        for play_mode_index in 0..crate::PLAY_MODE_COUNT {
            let mode_elapsed = if play_mode_index <= (play_mode as usize) {
                elapsed
            } else {
                std::time::Duration::from_secs(0)
            };

            self.play_mode_states[play_mode_index].update(mode_elapsed);
        }

        trace!(
            "fps: {:.1}  dt: {:.2}ms",
            self.play_mode_states[0].fps,
            self.play_mode_states[0].previous_frame_dt * 1000.0
        );
        if self.play_mode_states[0].previous_frame_dt > 1.0 / 30.0 {
            //warn!("slow frame (dt: {:.2}ms)", dt);
        }
    }

    pub fn system(&self) -> &ModeTimeState {
        &self.play_mode_states[PlayMode::System as usize]
    }

    pub fn paused(&self) -> &ModeTimeState {
        &self.play_mode_states[PlayMode::Paused as usize]
    }

    pub fn playing(&self) -> &ModeTimeState {
        &self.play_mode_states[PlayMode::Playing as usize]
    }
}

#[derive(Copy, Clone)]
pub struct ModeTimeState {
    // Duration of time passed since app_start_system_time
    pub total_time: time::Duration,

    // rust Instant object captured at the start of the frame
    pub frame_start_instant: time::Instant,

    // duration of time passed during the previous frame
    pub previous_frame_time: time::Duration,

    pub previous_frame_dt: f32,

    pub fps: f32,

    pub fps_smoothed: f32,

    pub frame_count: u64,
}

impl ModeTimeState {
    pub fn new() -> Self {
        let now_instant = time::Instant::now();
        let zero_duration = time::Duration::from_secs(0);
        return ModeTimeState {
            total_time: zero_duration,
            frame_start_instant: now_instant,
            previous_frame_time: zero_duration,
            previous_frame_dt: 0.0,
            fps: 0.0,
            fps_smoothed: 0.0,
            frame_count: 0,
        };
    }

    pub fn update(&mut self, elapsed: std::time::Duration) {
        self.total_time += elapsed;
        self.frame_start_instant += elapsed;
        self.previous_frame_time = elapsed;

        // this can eventually be replaced with as_float_secs
        let dt =
            (elapsed.as_secs() as f32) + (elapsed.subsec_nanos() as f32) / (NANOS_PER_SEC as f32);

        self.previous_frame_dt = dt;

        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        //TODO: Replace with a circular buffer
        const SMOOTHING_FACTOR: f32 = 0.95;
        self.fps = fps;
        self.fps_smoothed =
            (self.fps_smoothed * SMOOTHING_FACTOR) + (fps * (1.0 - SMOOTHING_FACTOR));

        self.frame_count += 1;
    }
}
