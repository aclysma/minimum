use rendy::wsi::winit;

use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ResourceTaskImpl, TaskConfig};

use framework::resources::FrameworkActionQueue;

use crate::resources::ImguiManager;
use crate::resources::{InputManager, WindowInterface};
use named_type::NamedType;

#[derive(NamedType)]
pub struct GatherInput;
pub type GatherInputTask = minimum::ResourceTask<GatherInput>;
impl ResourceTaskImpl for GatherInput {
    type RequiredResources = (
        Read<winit::window::Window>,
        Read<WindowInterface>,
        Write<ImguiManager>,
        Write<InputManager>,
        Write<FrameworkActionQueue>,
    );
    //const REQUIRED_FLAGS: usize = framework::context_flags::AUTHORITY_CLIENT as usize
    //    | framework::context_flags::PLAYMODE_SYSTEM as usize;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseGatherInput>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        use winit::event::Event;
        use winit::event::WindowEvent;

        let (window, window_interface, mut imgui_manager, mut input_manager, mut framework_action_queue) =
            data;

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
            framework_action_queue.enqueue_terminate_process();
        }
    }
}
