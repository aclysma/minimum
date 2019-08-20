use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, EntitySet, ReadComponent, Task, TaskContext};

use crate::resources::{
    DebugOptions, GameControl, ImguiManager, InputManager, PhysicsManager, RenderState, TimeState,
};

use crate::components::{BulletComponent, PlayerComponent, PositionComponent};
use named_type::NamedType;

#[derive(NamedType)]
pub struct RenderImguiMainMenu;
impl Task for RenderImguiMainMenu {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Write<GameControl>,
        Write<DebugOptions>,
        Read<PhysicsManager>,
        Read<EntitySet>,
        ReadComponent<BulletComponent>,
        ReadComponent<PlayerComponent>,
        ReadComponent<PositionComponent>,
        Read<InputManager>,
        Read<RenderState>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            time_state,
            mut imgui_manager,
            mut _game_control,
            mut debug_options,
            physics_manager,
            entity_set,
            bullet_components,
            player_components,
            position_components,
            input_manager,
            render_state,
        ) = data;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

//            ui.window(im_str!("Editor")).build(|| {
//                let mut b = true;
//                ui.show_metrics_window(&mut b);
//                ui.show_default_style_editor();
//            });

            let paused = time_state.play_mode == crate::PlayMode::System;

            {
                let _style_token = if paused {
                    Some(ui.push_style_color(imgui::StyleColor::MenuBarBg, [0.2, 0.2, 0.2, 1.0]))
                } else {
                    None
                };

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

                    ui.text(im_str!("FPS: {:.1}", time_state.system().fps_smoothed));
                    ui.separator();
                });
            }

            if debug_options.show_window {
                let bullet_count = bullet_components.count();
                let mouse_position_ui_space = input_manager.mouse_position();
                let mouse_position_world_space = render_state.ui_space_to_world_space(glm::vec2(
                    mouse_position_ui_space.x as f32,
                    mouse_position_ui_space.y as f32,
                ));
                let body_count = physics_manager.world().bodies().count();

                let mut player_position = None;
                for (entity_handle, _player) in player_components.iter(&entity_set) {
                    if let Some(position) = position_components.get(&entity_handle) {
                        player_position = Some(position.position());
                    }
                    break;
                }

                ui.window(im_str!("Debug Window")).build(|| {
                    if let Some(p) = player_position {
                        ui.text(format!("player world space: {:.1} {:.1}", p.x, p.y));
                    }
                    ui.text(format!(
                        "mouse screen space: {:.1} {:.1}",
                        mouse_position_ui_space.x, mouse_position_ui_space.y
                    ));
                    ui.text(format!(
                        "mouse world space: {:.1} {:.1}",
                        mouse_position_world_space.x, mouse_position_world_space.y
                    ));
                    ui.text(format!("bullet count: {}", bullet_count));
                    ui.text(format!("body count: {}", body_count));
                });

                //TODO: Component count
                //TODO: Frame time
            }
        })
    }
}
