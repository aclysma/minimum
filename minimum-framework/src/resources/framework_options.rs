pub struct FrameworkOptions {
    pub show_debug_info: bool,
}

impl FrameworkOptions {
    pub fn new() -> Self {
        FrameworkOptions {
            show_debug_info: false,
        }
    }
}
