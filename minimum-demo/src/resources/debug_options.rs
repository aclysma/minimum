pub struct DebugOptions {
    pub show_window: bool,
    pub show_imgui_metrics: bool,
    pub show_imgui_style_editor: bool,
}

impl DebugOptions {
    pub fn new() -> Self {
        DebugOptions {
            show_window: false,
            show_imgui_metrics: false,
            show_imgui_style_editor: false,
        }
    }
}
