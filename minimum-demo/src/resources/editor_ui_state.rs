use imgui::ImString;
use crate::framework::inspect::InspectorTab;

pub struct WindowOptions {
    pub show_imgui_metrics: bool,
    pub show_imgui_style_editor: bool,
    pub show_imgui_demo: bool,
    pub show_entity_list: bool,
    pub show_inspector: bool,
}

impl WindowOptions {
    pub fn new() -> Self {
        WindowOptions {
            show_imgui_metrics: false,
            show_imgui_style_editor: false,
            show_imgui_demo: false,
            show_entity_list: false,
            show_inspector: false,
        }
    }

    pub fn new_runtime() -> Self {
        let mut options = Self::new();
        options.show_entity_list = true;
        options.show_inspector = true;
        options
    }

    pub fn new_editing() -> Self {
        let mut options = Self::new();
        options.show_entity_list = true;
        options.show_inspector = true;
        options
    }
}

// If adding to this, don't forget to hook up keyboard shortcuts and buttons
#[derive(PartialEq)]
pub enum EditorTool {
    Select,
    Translate,
    Rotate,
    Scale,
}

pub struct EditorUiState {
    pub add_component_search_text: ImString,
    pub window_options_running: WindowOptions,
    pub window_options_editing: WindowOptions,
    pub set_inspector_tab: Option<InspectorTab>,
    pub active_editor_tool: EditorTool
}

impl EditorUiState {
    pub fn new() -> Self {
        EditorUiState {
            add_component_search_text: ImString::with_capacity(255),
            window_options_running: WindowOptions::new_runtime(),
            window_options_editing: WindowOptions::new_editing(),
            set_inspector_tab: None,
            active_editor_tool: EditorTool::Select
        }
    }

    pub fn window_options(&self, play_mode: crate::PlayMode) -> &WindowOptions {
        if play_mode == crate::PlayMode::System {
            &self.window_options_editing
        } else {
            &self.window_options_running
        }
    }

    pub fn window_options_mut(&mut self, play_mode: crate::PlayMode) -> &mut WindowOptions {
        if play_mode == crate::PlayMode::System {
            &mut self.window_options_editing
        } else {
            &mut self.window_options_running
        }
    }
}
