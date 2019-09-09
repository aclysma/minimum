use minimum::component::SlabComponentStorage;

use framework::resources::TimeState;

#[derive(Debug, Clone)]
pub struct FreeAtTimeComponent {
    free_time: std::time::Instant,
}

impl FreeAtTimeComponent {
    pub fn new(free_time: std::time::Instant) -> Self {
        FreeAtTimeComponent { free_time }
    }

    pub fn should_free(&self, time_state: &TimeState) -> bool {
        time_state.playing().frame_start_instant >= self.free_time
    }
}

impl minimum::Component for FreeAtTimeComponent {
    type Storage = SlabComponentStorage<Self>;
}
