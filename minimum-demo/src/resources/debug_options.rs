pub struct DebugOptions {
    pub show_debug_info: bool,
}

impl DebugOptions {
    pub fn new() -> Self {
        DebugOptions {
            show_debug_info: false,
        }
    }
}
