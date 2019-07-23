
use minimum::systems::{DataRequirement, Read, ReadOption, async_dispatch::Task, Write};

use crate::resources::{
    ImguiManager,
    WindowInterface,
    InputManager,
    GameControl,
    TimeState,
    DebugDraw
};

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
        Read<TimeState>,
        Write<ImguiManager>,
        Write<GameControl>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            time_state,
            mut imgui_manager,
            mut game_control,
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

