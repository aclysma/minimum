use minimum::input::KeyboardKey;
use minimum::input::ButtonState;

pub use winit::event::VirtualKeyCode;
pub use winit::event::ElementState;
pub use winit::event::MouseScrollDelta;
pub use winit::event::MouseButton;
use minimum::resources::ViewportResource;

#[derive(Copy, Clone)]
pub struct WinitKeyboardKey {
    keycode: VirtualKeyCode,
}

impl WinitKeyboardKey {
    pub fn new(keycode: VirtualKeyCode) -> Self {
        WinitKeyboardKey { keycode }
    }
}

impl Into<KeyboardKey> for WinitKeyboardKey {
    fn into(self) -> KeyboardKey {
        KeyboardKey(self.keycode as u8)
    }
}

#[derive(Copy, Clone)]
pub struct WinitElementState {
    element_state: ElementState,
}

impl WinitElementState {
    pub fn new(element_state: ElementState) -> Self {
        WinitElementState { element_state }
    }
}

impl Into<ButtonState> for WinitElementState {
    fn into(self) -> ButtonState {
        match self.element_state {
            ElementState::Pressed => ButtonState::Pressed,
            ElementState::Released => ButtonState::Released,
        }
    }
}

#[derive(Copy, Clone)]
pub struct WinitMouseButton {
    mouse_button: MouseButton,
}

impl WinitMouseButton {
    pub fn new(mouse_button: MouseButton) -> Self {
        WinitMouseButton { mouse_button }
    }
}

impl Into<minimum::input::MouseButton> for WinitMouseButton {
    fn into(self) -> minimum::input::MouseButton {
        let button_index = match self.mouse_button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => x + 3,
        };

        minimum::input::MouseButton(button_index)
    }
}

#[derive(Copy, Clone)]
pub struct WinitMouseScrollDelta {
    mouse_scroll_delta: MouseScrollDelta,
}

impl WinitMouseScrollDelta {
    pub fn new(mouse_scroll_delta: MouseScrollDelta) -> Self {
        WinitMouseScrollDelta { mouse_scroll_delta }
    }
}

impl Into<minimum::input::MouseScrollDelta> for WinitMouseScrollDelta {
    fn into(self) -> minimum::input::MouseScrollDelta {
        let delta = match self.mouse_scroll_delta {
            MouseScrollDelta::LineDelta(x, y) => (x, y),
            MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
        };

        minimum::input::MouseScrollDelta {
            x: delta.0,
            y: delta.1,
        }
    }
}

/// Call when winit sends an event
pub fn handle_winit_event<T>(
    event: &winit::event::Event<T>,
    input_state: &mut minimum::input::InputState,
    viewport: &ViewportResource,
) {
    use winit::event::Event;
    use winit::event::WindowEvent;

    let _is_close_requested = false;

    match event {
        //Process keyboard input
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            trace!("keyboard input {:?}", input);
            if let Some(vk) = input.virtual_keycode {
                input_state.handle_keyboard_event(
                    WinitKeyboardKey::new(vk).into(),
                    WinitElementState::new(input.state).into(),
                );
            }
        }

        Event::WindowEvent {
            event:
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                },
            ..
        } => {
            trace!(
                "mouse button input {:?} {:?} {:?}",
                device_id,
                state,
                button,
            );

            input_state.handle_mouse_button_event(
                WinitMouseButton::new(*button).into(),
                WinitElementState::new(*state).into(),
                viewport,
            );
        }

        Event::WindowEvent {
            event:
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                },
            ..
        } => {
            trace!("mouse move input {:?} {:?}", device_id, position,);
            input_state.handle_mouse_move_event(
                glam::Vec2::new(position.x as f32, position.y as f32),
                viewport,
            );
        }

        Event::WindowEvent {
            event: WindowEvent::MouseWheel {
                device_id, delta, ..
            },
            ..
        } => {
            trace!("mouse wheel {:?} {:?}", device_id, delta);
            input_state.handle_mouse_wheel_event(WinitMouseScrollDelta::new(*delta).into());
        }

        // Ignore any other events
        _ => (),
    }
}
