use minimum_game::input::KeyboardKey;

pub struct Keybinds {
    pub selection_add: KeyboardKey,
    pub selection_subtract: KeyboardKey,
    pub selection_toggle: KeyboardKey,

    pub tool_translate: KeyboardKey,
    pub tool_scale: KeyboardKey,
    pub tool_rotate: KeyboardKey,

    pub action_quit: KeyboardKey,
    pub action_toggle_editor_pause: KeyboardKey,
}

pub struct EditorSettingsResource {
    keybinds: Keybinds,
}

impl EditorSettingsResource {
    pub fn new(keybinds: Keybinds) -> Self {
        EditorSettingsResource { keybinds }
    }

    pub fn keybinds(&self) -> &Keybinds {
        &self.keybinds
    }
}
