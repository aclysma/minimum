pub struct FpsTextResource {
    pub last_fps_text_change: Option<std::time::Instant>,
    pub fps_text: String,
}

impl FpsTextResource {
    pub fn new() -> Self {
        FpsTextResource {
            last_fps_text_change: None,
            fps_text: "".to_string(),
        }
    }
}
