pub struct DebugOptions {
    pub show_window: bool,
}

impl DebugOptions {
    pub fn new() -> Self {
        DebugOptions { show_window: false }
    }
}
