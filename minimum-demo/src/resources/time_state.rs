use std::time;

const NANOS_PER_SEC: u32 = 1_000_000_000;

// This is not intended to be accessed when the system time updates, but we can double buffer it
// if it becomes a problem
pub struct TimeState {
    //
    // Will not change once set:
    //

    // System time that the application started
    pub app_start_system_time: time::SystemTime,

    // rust Instant object captured when the application started
    pub app_start_instant: time::Instant,

    //
    // These will change every frame
    //

    // rust Instant object captured at the start of the frame
    pub frame_start_instant: time::Instant,

    // Duration of time passed since app_start_system_time
    pub app_duration: time::Duration,

    // duration of time passed during the previous frame
    pub previous_frame_time: time::Duration,

    pub previous_frame_dt: f32,

    pub fps: f32,

    pub fps_smoothed: f32,

    pub frame_count: u64,
}

impl TimeState {
    pub fn new() -> TimeState {
        let now_instant = time::Instant::now();
        let now_system_time = time::SystemTime::now();
        let zero_duration = time::Duration::from_secs(0);

        return TimeState {
            app_start_system_time: now_system_time,
            app_start_instant: now_instant,
            frame_start_instant: now_instant,
            app_duration: zero_duration,
            previous_frame_time: zero_duration,
            previous_frame_dt: 0.0,
            fps: 0.0,
            fps_smoothed: 0.0,
            frame_count: 0,
        };
    }
    //(self.secs as f64) + (self.nanos as f64) / (NANOS_PER_SEC as f64)
    pub fn update(&mut self) {
        let now_instant = time::Instant::now();

        self.previous_frame_time = now_instant - self.frame_start_instant;
        self.frame_start_instant = now_instant;
        self.app_duration = time::Instant::now() - self.app_start_instant;

        // this can eventually be replaced with as_float_secs
        let dt = self.previous_frame_time;
        let dt = (dt.as_secs() as f32) + (dt.subsec_nanos() as f32) / (NANOS_PER_SEC as f32);

        self.previous_frame_dt = dt;

        let dt = self.previous_frame_dt;
        let fps = if self.previous_frame_dt > 0.0 {
            1.0 / self.previous_frame_dt
        } else {
            0.0
        };

        //TODO: Replace with a circular buffer
        const SMOOTHING_FACTOR: f32 = 0.95;
        self.fps = fps;
        self.fps_smoothed =
            (self.fps_smoothed * SMOOTHING_FACTOR) + (fps * (1.0 - SMOOTHING_FACTOR));

        self.frame_count += 1;

        trace!("fps: {:.1}  dt: {:.2}ms", self.fps, dt * 1000.0);
        if dt > 1.0 / 30.0 {
            //warn!("slow frame (dt: {:.2}ms)", dt);
        }
    }
}
