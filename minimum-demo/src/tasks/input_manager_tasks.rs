use minimum::systems::{async_dispatch::Task, DataRequirement, Read, Write};

use crate::resources::{GameControl, ImguiManager, InputManager, WindowInterface};

#[derive(typename::TypeName)]
pub struct GatherInput;
impl Task for GatherInput {
    type RequiredResources = (
        Read<winit::window::Window>,
        Read<WindowInterface>,
        Write<ImguiManager>,
        Write<InputManager>,
        Write<GameControl>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        use winit::event::Event;
        use winit::event::WindowEvent;

        let (window, window_interface, mut imgui_manager, mut input_manager, mut game_control) =
            data;

        input_manager.clear_events_from_previous_frame();
        let mut is_close_requested = false;

        let imgui_want_capture_keyboard = imgui_manager.want_capture_keyboard();
        let imgui_want_capture_mouse = imgui_manager.want_capture_mouse();

        loop {
            match window_interface.event_rx.lock().unwrap().try_recv() {
                Ok(event) => {
                    imgui_manager.handle_event(&window, &event);

                    match event {
                        // Close if the window is killed
                        Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            ..
                        } => is_close_requested = true,

                        // Close if the escape key is hit
                        Event::WindowEvent {
                            event:
                                WindowEvent::KeyboardInput {
                                    input:
                                        winit::event::KeyboardInput {
                                            virtual_keycode:
                                                Some(winit::event::VirtualKeyCode::Escape),
                                            ..
                                        },
                                    ..
                                },
                            ..
                        } => is_close_requested = true,

                        //Process keyboard input
                        Event::WindowEvent {
                            event: WindowEvent::KeyboardInput { input, .. },
                            ..
                        } => {
                            trace!("keyboard {:?}", input);
                            if !imgui_want_capture_keyboard {
                                input_manager.handle_keyboard_event(&input);
                            }
                        }

                        Event::WindowEvent {
                            event:
                                WindowEvent::MouseInput {
                                    device_id,
                                    state,
                                    button,
                                    modifiers,
                                },
                            ..
                        } => {
                            trace!(
                                "mouse {:?} {:?} {:?} {:?}",
                                device_id,
                                state,
                                button,
                                modifiers
                            );
                            if !imgui_want_capture_mouse {
                                input_manager.handle_mouse_button_event(state, button, modifiers);
                            }
                        }

                        Event::WindowEvent {
                            event:
                                WindowEvent::CursorMoved {
                                    device_id,
                                    position,
                                    modifiers,
                                },
                            ..
                        } => {
                            trace!("mouse {:?} {:?} {:?}", device_id, position, modifiers);
                            input_manager.handle_mouse_move_event(position);
                        }

                        // Ignore any other events
                        _ => (),
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("winit thread failed"),
            }
        }

        if is_close_requested {
            game_control.set_terminate_process();
        }
    }
}
