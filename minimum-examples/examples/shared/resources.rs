
// A resource that keeps track of how many times the update loop has run
pub struct UpdateCount {
    pub count: i32,
}

impl UpdateCount {
    pub fn new() -> Self {
        return UpdateCount { count: 0 };
    }
}

// A resource that keeps track of frame dt
pub struct TimeState {
    pub dt: f32,
}

impl TimeState {
    pub fn new() -> Self {
        TimeState { dt: 1.0 / 60.0 }
    }
}
