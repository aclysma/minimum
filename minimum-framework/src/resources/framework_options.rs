use crate::resources::KeyboardButton;

pub struct FrameworkKeybinds {
    pub edit_play_toggle: KeyboardButton,
    pub translate_tool: KeyboardButton,
    pub scale_tool: KeyboardButton,
    pub rotate_tool: KeyboardButton,
    pub quit: KeyboardButton,
    pub modify_selection_add1: KeyboardButton,
    pub modify_selection_add2: KeyboardButton,
    pub modify_selection_subtract1: KeyboardButton,
    pub modify_selection_subtract2: KeyboardButton,
    pub modify_imgui_entity_list_modify_selection_add1: KeyboardButton,
    pub modify_imgui_entity_list_modify_selection_add2: KeyboardButton,
    pub clear_selection: KeyboardButton
}

pub struct FrameworkOptions {
    pub show_debug_info: bool,
    pub keybinds: FrameworkKeybinds

}

impl FrameworkOptions {
    pub fn new(keybinds: FrameworkKeybinds) -> Self {
        FrameworkOptions {
            show_debug_info: false,
            keybinds
        }
    }
}
