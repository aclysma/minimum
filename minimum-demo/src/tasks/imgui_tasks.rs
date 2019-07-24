
use minimum::systems::{DataRequirement, Read, ReadOption, async_dispatch::Task, Write};
use minimum::{Component, EntitySet, ComponentStorage};

use crate::resources::{
    ImguiManager,
    WindowInterface,
    InputManager,
    GameControl,
    TimeState,
    DebugDraw,
    DebugOptions,
    RenderState
};

use crate::components::{
    BulletComponent,
    PlayerComponent,
    PositionComponent
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
        Write<DebugOptions>,
        Read<EntitySet>,
        Read<<BulletComponent as Component>::Storage>,
        Read<<PlayerComponent as Component>::Storage>,
        Read<<PositionComponent as Component>::Storage>,
        Read<InputManager>,
        Read<RenderState>
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            time_state,
            mut imgui_manager,
            mut game_control,
            mut debug_options,
            entity_set,
            bullet_components,
            player_components,
            position_components,
            input_manager,
            render_state
        ) = data;

        imgui_manager.with_ui(|ui : &mut imgui::Ui| {
            use imgui::im_str;

            ui.main_menu_bar(|| {
                ui.menu(im_str!("File")).build(|| {
                    ui.menu(im_str!("Load")).build(|| {
                        for pack in &["placeholder1", "placeholder2", "placeholder3"] {
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

                ui.checkbox(im_str!("Debug Window"), &mut debug_options.show_window);
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

            if debug_options.show_window {
                //let mut demo_window_opened = true;
                //ui.show_demo_window(&mut debug_options.show_window);

                let bullet_count = bullet_components.count();

                let mouse_position_ui_space = input_manager.mouse_position();

                let mouse_position_world_space = render_state.ui_space_to_world_space(glm::vec2(mouse_position_ui_space.x as f32, mouse_position_ui_space.y as f32));

                let mut player_position = None;
                for (entity_handle, player) in player_components.iter(&entity_set) {
                    if let Some(position) = position_components.get(&entity_handle) {
                        player_position = Some(position.position());
                    }
                    break;
                }

                ui.window(im_str!("Debug Window"))
                    .build(|| {

                        if let Some(p) = player_position {
                            ui.text(format!("player world space: {:.1} {:.1}", p.x, p.y));
                        }
                        ui.text(format!("mouse screen space: {:.1} {:.1}", mouse_position_ui_space.x, mouse_position_ui_space.y));
                        ui.text(format!("mouse world space: {:.1} {:.1}", mouse_position_world_space.x, mouse_position_world_space.y));
                        ui.text(format!("bullet count: {}", bullet_count));
                    });
            }
        })
    }
}

