use winit::event::KeyboardInput;
use winit::event::WindowEvent;
use winit::dpi::LogicalPosition;

pub struct InputManager {
    key_states_is_down: [bool; 255],
    key_states_just_down: [bool; 255],

    mouse_buttons_down: [bool; 3],
    mouse_buttons_just_down: [bool; 3],
    mouse_position: winit::dpi::LogicalPosition
}

pub enum MouseButtons {
    Left,
    Middle,
    Right
}

impl InputManager {
    pub fn new() -> InputManager {
        return InputManager {
            key_states_is_down: [false; 255],
            key_states_just_down: [false; 255],
            mouse_buttons_down: [false; 3],
            mouse_buttons_just_down: [false; 3],
            mouse_position: LogicalPosition { x: 0.5, y: 0.5 }
        };
    }

    pub fn is_key_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_states_is_down[key as usize];
    }

    pub fn is_key_just_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_states_just_down[key as usize];
    }

    pub fn mouse_position(&self) -> LogicalPosition {
        return self.mouse_position
    }

    pub fn is_mouse_down(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_buttons_down[mouse_button as usize];
    }

    pub fn is_mouse_just_down(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_buttons_just_down[mouse_button as usize];
    }
}

impl InputManager {
    pub fn clear_events_from_previous_frame(&mut self) {
        for value in self.key_states_just_down.iter_mut() {
            *value = false;
        }

        for value in self.mouse_buttons_just_down.iter_mut() {
            *value = false;
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &KeyboardInput) {
        //TODO: Find a safer way to change enum back/forth with int
        // Assign true if key is down, or false if key is up
        if let Some(kc) = event.virtual_keycode {
            if kc as u32 > 255 {
                error!("kc {} out of expected range", kc as u32);
            }

            //TODO: Handle repeating keys (blocked by https://github.com/rust-windowing/winit/issues/753)
            self.key_states_is_down[kc as usize] = match event {
                KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    ..
                } => {
                    if !self.key_states_is_down[kc as usize] {
                        self.key_states_just_down[kc as usize] = true;
                    }
                    true
                }
                KeyboardInput {
                    state: winit::event::ElementState::Released,
                    ..
                } => false,
            };
        }
    }

    pub fn handle_mouse_button_event(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
        modifiers: winit::event::ModifiersState
    ) {
        use winit::event::MouseButton;
        use winit::event::ElementState;

        let button_index : i32 = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            _ => -1
        };

        if button_index < 0 {
            return;
        }

        let button_index = button_index as usize;

        if state == winit::event::ElementState::Pressed {
            self.mouse_buttons_just_down[button_index] = true;
        }

        self.mouse_buttons_down[button_index] = match state {
            ElementState::Pressed => true,
            ElementState::Released => false
        };
    }

    pub fn handle_mouse_move_event(
        &mut self,
        position: winit::dpi::LogicalPosition
    ) {
        self.mouse_position = position;
    }
}
