use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::mouse::MouseWheelDirection;

use sdl2::event::Event;

use minimum::resources::ViewportResource;
use minimum::input::InputState;
//use minimum::input::KeyboardKey;
//use minimum::input::MouseButton;

use minimum::input as minimum_input;

#[derive(Copy, Clone)]
pub struct Sdl2KeyboardKey {
    keycode: Keycode,
}

impl Sdl2KeyboardKey {
    pub fn new(keycode: Keycode) -> Self {
        Sdl2KeyboardKey { keycode }
    }
}

impl Into<minimum_input::KeyboardKey> for Sdl2KeyboardKey {
    fn into(self) -> minimum_input::KeyboardKey {
        minimum_input::KeyboardKey(self.keycode as u8)
    }
}

//#[derive(Copy, Clone)]
//pub struct Sdl2ElementState {
//    element_state: ElementState,
//}
//
//impl WinitElementState {
//    pub fn new(element_state: ElementState) -> Self {
//        WinitElementState { element_state }
//    }
//}
//
//impl Into<ButtonState> for WinitElementState {
//    fn into(self) -> ButtonState {
//        match self.element_state {
//            ElementState::Pressed => ButtonState::Pressed,
//            ElementState::Released => ButtonState::Released,
//        }
//    }
//}

#[derive(Copy, Clone)]
pub struct Sdl2MouseButton {
    mouse_button: MouseButton,
}

impl Sdl2MouseButton {
    pub fn new(mouse_button: MouseButton) -> Self {
        Sdl2MouseButton { mouse_button }
    }
}

impl Into<minimum_input::MouseButton> for Sdl2MouseButton {
    fn into(self) -> minimum_input::MouseButton {
        let button_index = match self.mouse_button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::X1 => 3,
            MouseButton::X2 => 4,
            MouseButton::Unknown => 5,
        };

        minimum_input::MouseButton(button_index)
    }
}

//#[derive(Copy, Clone)]
//pub struct WinitMouseScrollDelta {
//    mouse_scroll_delta: MouseScrollDelta,
//}
//
//impl WinitMouseScrollDelta {
//    pub fn new(mouse_scroll_delta: MouseScrollDelta) -> Self {
//        WinitMouseScrollDelta { mouse_scroll_delta }
//    }
//}
//
//impl Into<minimum::input::MouseScrollDelta> for WinitMouseScrollDelta {
//    fn into(self) -> minimum::input::MouseScrollDelta {
//        let delta = match self.mouse_scroll_delta {
//            MouseScrollDelta::LineDelta(x, y) => (x, y),
//            MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
//        };
//
//        minimum::input::MouseScrollDelta {
//            x: delta.0,
//            y: delta.1,
//        }
//    }
//}

/// Call when winit sends an event
pub fn handle_sdl2_event(
    event: &Event,
    input_state: &mut InputState,
    viewport: &ViewportResource,
) {
    let mut is_close_requested = false;

    match event {
        Event::KeyDown { keycode, repeat, .. } => {
            handle_keyboard_event(input_state, keycode, minimum_input::ButtonState::Pressed)
        },
        Event::KeyUp { keycode, repeat, .. } => {
            handle_keyboard_event(input_state, keycode, minimum_input::ButtonState::Released)
        },
        Event::MouseButtonDown { mouse_btn, .. } => {
            handle_mouse_button_event(input_state, mouse_btn, viewport, minimum_input::ButtonState::Pressed)
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            handle_mouse_button_event(input_state, mouse_btn, viewport, minimum_input::ButtonState::Released)
        },
        Event::MouseMotion { x, y, .. } => {
            input_state.handle_mouse_move_event(
                glam::Vec2::new(*x as f32, *y as f32),
                viewport
            );
        },
        Event::MouseWheel { x, y, .. }=> {
            input_state.handle_mouse_wheel_event(
                minimum_input::MouseScrollDelta::new(*x as f32, *y as f32)
            );
        }

        // Ignore any other events
        _ => (),
    }
}

fn handle_mouse_button_event(input_state: &mut InputState, mouse_btn: &MouseButton, viewport: &ViewportResource, button_state: minimum_input::ButtonState) -> () {
    input_state.handle_mouse_button_event(
        Sdl2MouseButton::new(*mouse_btn).into(),
        button_state,
        viewport
    )
}

fn handle_keyboard_event(input_state: &mut InputState, keycode: &Option<Keycode>, button_state: minimum_input::ButtonState) -> () {
    if let Some(kc) = keycode {
        input_state.handle_keyboard_event(
            Sdl2KeyboardKey::new(*kc).into(),
            button_state
        )
    }
}
