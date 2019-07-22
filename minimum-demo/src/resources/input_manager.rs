use winit::event::KeyboardInput;

pub struct InputManager {
    // this is not intended to be accessed when pumping window messages that mutate state,
    // but we can always double buffer it if needed
    //
    // handle_keyboard_event is non-mutable but needs to change the local state.
    // Cell<bool> is a zero-cost abstraction, and it's relatively safe for us to poke
    // values into it, even in a multi-threaded scenario (although this isn't intended)
    // Wanted this to be an array but had trouble with making a large array.. probably
    // not a bad thing to box it to get it into a separate heap alloc though since in
    // theory this could be large
    //
    // This could have also been a RefCell but I want to avoid the overhead. Could also
    // be a swapped double-buffer of these states.. that might be nicer for the borrow
    // checker, but I think this will be fine
    key_states_is_down: [bool; 255],
    key_states_just_down: [bool; 255],
}

impl InputManager {
    pub fn new() -> InputManager {
        // This is a bit ugly, I would prefer this to be an array but rust doesn't
        // have a good way to initialize an array of non-copy items, and cells are non-copy

        return InputManager {
            key_states_is_down: [false; 255],
            key_states_just_down: [false; 255],
        };
    }

    pub fn is_key_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_states_is_down[key as usize];
    }

    pub fn is_key_just_down(&self, key: winit::event::VirtualKeyCode) -> bool {
        return self.key_states_just_down[key as usize];
    }
}

impl InputManager {
    pub fn clear_events_from_previous_frame(&mut self) {
        for value in self.key_states_just_down.iter_mut() {
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
            // Will be able to remove JUMP_DELAY_MILLIS
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
}
