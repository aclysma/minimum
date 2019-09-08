use rendy::wsi::winit;
use winit::event::KeyboardInput;

#[derive(Copy, Clone, Debug)]
pub struct MouseDragState {
    pub begin_position: glm::Vec2,
    pub end_position: glm::Vec2,
}

pub struct InputManager {
    key_is_down: [bool; 255],
    key_just_down: [bool; 255],
    key_just_up: [bool; 255],

    mouse_position: glm::Vec2,
    mouse_button_is_down: [bool; 3],
    mouse_button_just_down: [Option<glm::Vec2>; 3],
    mouse_button_just_up: [Option<glm::Vec2>; 3],

    mouse_button_just_clicked: [Option<glm::Vec2>; 3],

    mouse_button_went_down_position: [Option<glm::Vec2>; 3],
    mouse_button_went_up_position: [Option<glm::Vec2>; 3],

    mouse_drag_in_progress: [Option<MouseDragState>; 3],
    mouse_drag_just_finished: [Option<MouseDragState>; 3],
}

#[derive(strum_macros::EnumCount)]
pub enum MouseButtons {
    Left,
    Middle,
    Right,
}

impl InputManager {
    pub fn new() -> InputManager {
        return InputManager {
            key_is_down: [false; 255],
            key_just_down: [false; 255],
            key_just_up: [false; 255],
            mouse_position: glm::zero(),
            mouse_button_is_down: [false; MOUSEBUTTONS_COUNT],
            mouse_button_just_down: [None; MOUSEBUTTONS_COUNT],
            mouse_button_just_up: [None; MOUSEBUTTONS_COUNT],
            mouse_button_just_clicked: [None; MOUSEBUTTONS_COUNT],
            mouse_button_went_down_position: [None; MOUSEBUTTONS_COUNT],
            mouse_button_went_up_position: [None; MOUSEBUTTONS_COUNT],
            mouse_drag_in_progress: [None; MOUSEBUTTONS_COUNT],
            mouse_drag_just_finished: [None; MOUSEBUTTONS_COUNT],
        };
    }

    pub fn is_key_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_is_down[key as usize];
    }

    pub fn is_key_just_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_just_down[key as usize];
    }

    pub fn is_key_just_up(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_just_up[key as usize];
    }

    pub fn mouse_position(&self) -> glm::Vec2 {
        return self.mouse_position;
    }

    pub fn is_mouse_down(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_button_is_down[mouse_button as usize];
    }

    pub fn is_mouse_just_down(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_button_just_down[mouse_button as usize].is_some();
    }

    pub fn mouse_just_down_position(&self, mouse_button: MouseButtons) -> Option<glm::Vec2> {
        return self.mouse_button_just_down[mouse_button as usize];
    }

    pub fn is_mouse_just_up(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_button_just_up[mouse_button as usize].is_some();
    }

    pub fn mouse_just_up_position(&self, mouse_button: MouseButtons) -> Option<glm::Vec2> {
        return self.mouse_button_just_up[mouse_button as usize];
    }

    pub fn is_mouse_button_just_clicked(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_button_just_clicked[mouse_button as usize].is_some();
    }

    pub fn mouse_button_just_clicked_position(
        &self,
        mouse_button: MouseButtons,
    ) -> Option<glm::Vec2> {
        return self.mouse_button_just_clicked[mouse_button as usize];
    }

    pub fn mouse_button_went_down_position(&self, mouse_button: MouseButtons) -> Option<glm::Vec2> {
        return self.mouse_button_went_down_position[mouse_button as usize];
    }

    pub fn mouse_button_went_up_position(&self, mouse_button: MouseButtons) -> Option<glm::Vec2> {
        return self.mouse_button_went_up_position[mouse_button as usize];
    }

    pub fn is_mouse_drag_in_progress(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_drag_in_progress[mouse_button as usize].is_some();
    }

    pub fn mouse_drag_in_progress(&self, mouse_button: MouseButtons) -> Option<MouseDragState> {
        return self.mouse_drag_in_progress[mouse_button as usize];
    }

    pub fn is_mouse_drag_just_finished(&self, mouse_button: MouseButtons) -> bool {
        return self.mouse_drag_just_finished[mouse_button as usize].is_some();
    }

    pub fn mouse_drag_just_finished(&self, mouse_button: MouseButtons) -> Option<MouseDragState> {
        return self.mouse_drag_just_finished[mouse_button as usize];
    }
}

impl InputManager {
    pub fn pre_handle_events(&mut self) {
        for value in self.key_just_down.iter_mut() {
            *value = false;
        }

        for value in self.key_just_up.iter_mut() {
            *value = false;
        }

        for value in self.mouse_button_just_down.iter_mut() {
            *value = None;
        }

        for value in self.mouse_button_just_up.iter_mut() {
            *value = None;
        }

        for value in self.mouse_button_just_clicked.iter_mut() {
            *value = None;
        }

        for value in self.mouse_drag_just_finished.iter_mut() {
            *value = None;
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
            match event {
                KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    ..
                } => {
                    if !self.key_is_down[kc as usize] {
                        self.key_just_down[kc as usize] = true;
                    }
                    self.key_is_down[kc as usize] = true
                }
                KeyboardInput {
                    state: winit::event::ElementState::Released,
                    ..
                } => {
                    if self.key_is_down[kc as usize] {
                        self.key_just_up[kc as usize] = true;
                    }
                    self.key_is_down[kc as usize] = false
                }
            }
        }
    }

    pub fn handle_mouse_button_event(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
        _modifiers: winit::event::ModifiersState,
    ) {
        use winit::event::ElementState;
        use winit::event::MouseButton;

        let button_index: i32 = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            _ => -1,
        };

        if button_index < 0 {
            return;
        }

        let button_index = button_index as usize;

        // Update is down/up, just down/up
        match state {
            ElementState::Pressed => {
                self.mouse_button_just_down[button_index] = Some(self.mouse_position);
                self.mouse_button_is_down[button_index] = true;

                self.mouse_button_went_down_position[button_index] = Some(self.mouse_position);
            }
            ElementState::Released => {
                self.mouse_button_just_up[button_index] = Some(self.mouse_position);
                self.mouse_button_is_down[button_index] = false;

                self.mouse_button_went_up_position[button_index] = Some(self.mouse_position);

                match self.mouse_drag_in_progress[button_index] {
                    Some(in_progress) => {
                        self.mouse_drag_just_finished[button_index] = Some(MouseDragState {
                            begin_position: in_progress.begin_position,
                            end_position: self.mouse_position,
                        });
                    }
                    None => {
                        self.mouse_button_just_clicked[button_index] = Some(self.mouse_position)
                    }
                }

                self.mouse_drag_in_progress[button_index] = None;
            }
        }
    }

    pub fn handle_mouse_move_event(&mut self, position: winit::dpi::LogicalPosition) {
        //let old_mouse_position = self.mouse_position;

        // Update mouse position
        self.mouse_position = glm::vec2(position.x as f32, position.y as f32);

        // Update drag in progress state
        for i in 0..MOUSEBUTTONS_COUNT {
            if self.mouse_button_is_down[i] {
                self.mouse_drag_in_progress[i] = match self.mouse_drag_in_progress[i] {
                    None => {
                        match self.mouse_button_went_down_position[i] {
                            Some(went_down_position) => {
                                const MIN_DRAG_DISTANCE: f32 = 3.0;
                                let min_drag_distance_met =
                                    glm::distance(&went_down_position, &self.mouse_position)
                                        > MIN_DRAG_DISTANCE;
                                if min_drag_distance_met {
                                    // We dragged a non-trivial amount, start the drag
                                    Some(MouseDragState {
                                        begin_position: went_down_position,
                                        end_position: self.mouse_position,
                                    })
                                } else {
                                    // Mouse moved too small an amount to be considered a drag
                                    None
                                }
                            }

                            // We don't know where the mosue went down, so we can't start a drag
                            None => None,
                        }
                    }
                    Some(old_drag_state) => {
                        // We were already dragging, so just update the end position
                        Some(MouseDragState {
                            begin_position: old_drag_state.begin_position,
                            end_position: self.mouse_position,
                        })
                    }
                };
            }
        }
    }
}
