

use minimum::systems::{DataRequirement, Read, ReadOption, async_dispatch::Task, Write};

use crate::resources::{
    ImguiManager,
    WindowInterface,
    InputManager,
    GameControl,
    TimeState,
    DebugDraw
};

use crate::components;
use minimum::component::ComponentStorage;

pub struct ImguiBeginFrame;
impl Task for ImguiBeginFrame {
    type RequiredResources = (
        Read<winit::window::Window>,
        Write<ImguiManager>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (window, mut imgui_manager) = data;
        imgui_manager.begin_frame(&window);
    }
}

pub struct RenderImguiMainMenu;
impl Task for RenderImguiMainMenu {
    type RequiredResources = (
        //Read<gfx::Renderer>,
        //Read<LevelPackFileInfos>,
        Read<TimeState>,
        //ReadOption<game::GameState>,
        Write<ImguiManager>,
        Write<GameControl>,
        //Write<gfx::DebugCameraSettings>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            //renderer,
            //level_pack_file_infos,
            time_state,
            //game_state_option,
            mut imgui_manager,
            mut game_control,
            //mut debug_camera_settings,
        ) = data;

        imgui_manager.with_ui(|ui| {
            use imgui::im_str;

            ui.main_menu_bar(|| {
                ui.menu(im_str!("File")).build(|| {
                    ui.menu(im_str!("Load")).build(|| {
                        for pack in &["pack1", "pack2", "pack3"] {
                            ui.menu(&im_str!("{}", pack)).build(|| {
                                for level in &["level1", "level2", "level3"] {
                                    let selected = ui.menu_item(&im_str!("{}", level)).build();
                                    if selected {
                                        info!("Loading {} {}", pack, level);
                                        //game_control.set_load_level(level.path.clone());
                                    }
                                }
                            });
                        }
                    });
                });

                ui.separator();

//                let vore_position = if game_state_option.is_some() {
//                    game_state_option.unwrap().vore.position()
//                } else {
//                    glm::zero()
//                };

                //TODO: Would prefer to right-align this
                ui.text(im_str!("FPS: {:.1}", time_state.fps_smoothed));
                ui.separator();
//                ui.text(im_str!(
//                    "Pos: {:.1} {:.1}",
//                    vore_position.x,
//                    vore_position.y
//                ));
//                ui.separator();

//                let cam_pos = renderer.get_camera_position();
//                ui.text(im_str!("Cam: {:.1} {:.1}", cam_pos.x, cam_pos.y));
//                ui.text(im_str!("Zoom: {:.2}", renderer.get_camera_zoom()));
//
//                let mut is_debug_camera_enabled = debug_camera_settings.is_debug_camera_enabled();
//
//                ui.checkbox(im_str!("Debug Cam Enabled"), &mut is_debug_camera_enabled);
//
//                // If the debug camera is enabled and it gets unchecked, reset the camera
//                if debug_camera_settings.is_debug_camera_enabled() && !is_debug_camera_enabled {
//                    debug_camera_settings.clear_debug_camera();
//                }
            });

            let mut demo_window_opened = true;
            ui.show_demo_window(&mut demo_window_opened);
        })
    }
}

pub struct HandleInput;
impl Task for HandleInput {
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
                            trace!("print {:?}", input);
                            input_manager.handle_keyboard_event(&input);
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
            //dispatch_ctx.end_game_loop();
        }
    }
}

pub fn render(world: &minimum::systems::World) {
    let window = world.fetch::<winit::window::Window>();
    let mut renderer = world.fetch_mut::<crate::renderer::Renderer>();
    renderer.render(&window, &world);
}

pub struct UpdateTimeState;
impl Task for UpdateTimeState {
    type RequiredResources = (Write<TimeState>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut time_state = data;
        time_state.update();
    }
}


//pub struct UpdateDebugCameraSettings;
//impl Task for UpdateDebugCameraSettings {
//    type RequiredResources = (
//        Read<core::TimeState>,
//        Read<input::InputManager>,
//        Read<gfx::RenderState>,
//        Write<gfx::DebugCameraSettings>,
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (time_state, input_manager, render_state, mut debug_camera_settings) = data;
//        debug_camera_settings.update_debug_camera(&render_state, &input_manager, &time_state);
//    }
//}

//pub struct PrePhysics;
//impl Task for PrePhysics {
//    type RequiredResources = (
//        Read<input::InputManager>,
//        Read<core::TimeState>,
//        Write<game::GameState>,
//        Write<physics::Physics>,
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (input_manager, time_state, mut game_state, mut physics) = data;
//
//        game_state
//            .vore
//            .pre_physics_update(&input_manager, &time_state, &mut physics);
//    }
//}
//
//pub struct Physics;
//impl Task for Physics {
//    type RequiredResources = (Read<core::TimeState>, Write<physics::Physics>);
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (time_state, mut physics) = data;
//        physics.update(&time_state);
//    }
//}
//
//pub struct PostPhysics;
//impl Task for PostPhysics {
//    type RequiredResources = (
//        Write<physics::Physics>,
//        Write<game::GameState>,
//        Write<<crate::game::PickupComponent as minimum::component::Component>::Storage>
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (mut physics, mut game_state, mut pickups) = data;
//
//        game_state.vore.post_physics_update(&mut physics);
//
//        for pickup in pickups.iter_values_mut() {
//            pickup.post_physics_update(&mut physics);
//        }
//    }
//}

pub struct UpdateDebugDraw;
impl Task for UpdateDebugDraw {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        Read<<components::DebugDrawCircleComponent as minimum::component::Component>::Storage>,
        Read<<components::PositionComponent as minimum::component::Component>::Storage>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut debug_draw, entity_set, circle_components, position_components) = data;

        debug_draw.clear();

        for (entity_index, circle) in circle_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                debug_draw.add_circle(position.position(), circle.radius(), circle.color())
            }
        }
    }
}
