use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, DispatchControl, EntitySet, ReadComponent, Task, TaskContext};

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
        Write<DispatchControl>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            time_state,
            mut imgui_manager,
            mut game_control,
            mut debug_options,
            physics_manager,
            entity_set,
            bullet_components,
            player_components,
            position_components,
            input_manager,
            render_state,
            mut dispatch_control,
        ) = data;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

            if debug_options.show_imgui_metrics {
                ui.show_metrics_window(&mut debug_options.show_imgui_metrics);
            }

            if debug_options.show_imgui_style_editor {
                ui.window(im_str!("Editor")).build(|| {
                    ui.show_default_style_editor();
                });
            }

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

                    if ui.menu_item(im_str!("Save")).build() {
                        game_control.set_save_level(std::path::PathBuf::from("test_save"));
                    }
                });

                ui.menu(im_str!("Windows")).build(|| {
                    ui.checkbox(im_str!("Debug Window"), &mut debug_options.show_window);
                    ui.checkbox(
                        im_str!("ImGui Metrics"),
                        &mut debug_options.show_imgui_metrics,
                    );
                    ui.checkbox(
                        im_str!("ImGui Style Editor"),
                        &mut debug_options.show_imgui_style_editor,
                    );
                });

                ui.separator();

                let editing = time_state.play_mode == crate::PlayMode::System;
                if editing {
                    if ui.menu_item(im_str!("\u{f40a} Play")).build() {
                        // Clear playmode flags
                        *dispatch_control.next_frame_context_flags_mut() &=
                            !(crate::context_flags::PLAYMODE_SYSTEM
                                | crate::context_flags::PLAYMODE_PAUSED
                                | crate::context_flags::PLAYMODE_PLAYING);

                        // Set the appropriate ones
                        *dispatch_control.next_frame_context_flags_mut() |=
                            crate::context_flags::PLAYMODE_SYSTEM
                                | crate::context_flags::PLAYMODE_PAUSED
                                | crate::context_flags::PLAYMODE_PLAYING
                    }
                } else {
                    if ui.menu_item(im_str!("\u{f3e4} Pause")).build() {
                        *dispatch_control.next_frame_context_flags_mut() &=
                            !(crate::context_flags::PLAYMODE_SYSTEM
                                | crate::context_flags::PLAYMODE_PAUSED
                                | crate::context_flags::PLAYMODE_PLAYING);

                        // Set the appropriate ones
                        *dispatch_control.next_frame_context_flags_mut() |=
                            crate::context_flags::PLAYMODE_SYSTEM
                    }
                }

                ui.text(im_str!("FPS: {:.1}", time_state.system().fps_smoothed));
                ui.separator();
                ui.separator();
            });

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
