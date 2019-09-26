use rendy::wsi::winit;

use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::framework::resources::FrameworkActionQueue;

#[cfg(feature = "editor")]
use crate::resources::ImguiManager;
use crate::framework::resources::InputManager;
use crate::resources::WindowInterface;

pub struct GatherInput;
pub type GatherInputTask = crate::base::ResourceTask<GatherInput>;
impl ResourceTaskImpl for GatherInput {
    #[cfg(feature = "editor")]
    type RequiredResources = (
        Read<winit::window::Window>,
        Read<WindowInterface>,
        Write<ImguiManager>,
        Write<InputManager>,
        Write<FrameworkActionQueue>,
    );

    #[cfg(not(feature = "editor"))]
    type RequiredResources = (
        Read<WindowInterface>,
        Write<InputManager>,
        Write<FrameworkActionQueue>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseGatherInput>();
        config.run_only_if(crate::framework::context_flags::AUTHORITY_CLIENT);
        config.run_only_if(crate::framework::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use winit::event::Event;
        use winit::event::WindowEvent;

        #[cfg(feature = "editor")]
        let (
            window,
            window_interface,
            mut imgui_manager,
            mut input_manager,
            mut framework_action_queue,
        ) = data;

        #[cfg(not(feature = "editor"))]
        let (window_interface, mut input_manager, mut framework_action_queue) = data;

        input_manager.pre_handle_events();
        let mut is_close_requested = false;

        #[cfg(feature = "editor")]
        let imgui_want_capture_keyboard = imgui_manager.want_capture_keyboard();
        #[cfg(feature = "editor")]
        let imgui_want_capture_mouse = imgui_manager.want_capture_mouse();

        #[cfg(not(feature = "editor"))]
        let imgui_want_capture_keyboard = false;
        #[cfg(not(feature = "editor"))]
        let imgui_want_capture_mouse = false;

        loop {
            match window_interface.event_rx.lock().unwrap().try_recv() {
                Ok(event) => {
                    #[cfg(feature = "editor")]
                    {
                        imgui_manager.handle_event(&window, &event);
                    }

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

                                if let Some(vk) = input.virtual_keycode {
                                    let keyboard_button = crate::framework::resources::KeyboardButton::new(vk as u32);

                                    let keyboard_event = match input.state {
                                        winit::event::ElementState::Pressed => crate::framework::resources::KeyboardButtonEvent::Pressed,
                                        winit::event::ElementState::Released => crate::framework::resources::KeyboardButtonEvent::Released,
                                    };

                                    input_manager.handle_keyboard_event(keyboard_button, keyboard_event);
                                }
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

                            let mouse_button = match button {
                                winit::event::MouseButton::Left => Some(crate::framework::resources::MouseButton::Left),
                                winit::event::MouseButton::Right => Some(crate::framework::resources::MouseButton::Right),
                                winit::event::MouseButton::Middle => Some(crate::framework::resources::MouseButton::Middle),
                                _ => None
                            };

                            let mouse_event = match state {
                                winit::event::ElementState::Pressed => crate::framework::resources::MouseButtonEvent::Pressed,
                                winit::event::ElementState::Released => crate::framework::resources::MouseButtonEvent::Released,
                            };

                            if !imgui_want_capture_mouse {
                                if let Some(mouse_button) = mouse_button {
                                    input_manager.handle_mouse_button_event(mouse_button, mouse_event);
                                }
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

                            input_manager.handle_mouse_move_event(glm::vec2(position.x as f32, position.y as f32));
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
            framework_action_queue.enqueue_terminate_process();
        }
    }
}
