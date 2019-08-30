use minimum::resource::{DataRequirement, Read, Write};
use minimum::{Task, TaskContext};

use crate::resources::{DebugOptions, EditorUiState, FrameworkActionQueue, ImguiManager, TimeState};

use named_type::NamedType;

#[derive(NamedType)]
pub struct RenderImguiMainMenu;
impl Task for RenderImguiMainMenu {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Write<FrameworkActionQueue>,
        Write<EditorUiState>,
        Write<DebugOptions>,
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
            mut editor_ui_state,
            mut debug_options,
        ) = data;

        let is_edit_mode = time_state.play_mode == crate::PlayMode::System;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

            let window_settings = editor_ui_state.window_options_mut(time_state.play_mode);

            if window_settings.show_imgui_metrics {
                ui.show_metrics_window(&mut window_settings.show_imgui_metrics);
            }

            if window_settings.show_imgui_style_editor {
                ui.window(im_str!("Editor")).build(|| {
                    ui.show_default_style_editor();
                });
            }

            if window_settings.show_imgui_demo {
                ui.show_demo_window(&mut window_settings.show_imgui_demo);
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
                        game_control.enqueue_save_level(std::path::PathBuf::from("test_save"));
                    }
                });

                ui.menu(im_str!("Windows")).build(|| {
                    ui.checkbox(
                        im_str!("ImGui Metrics"),
                        &mut window_settings.show_imgui_metrics,
                    );
                    ui.checkbox(
                        im_str!("ImGui Style Editor"),
                        &mut window_settings.show_imgui_style_editor,
                    );
                    ui.checkbox(
                        im_str!("ImGui Demo"),
                        &mut window_settings.show_imgui_demo,
                    );
                    ui.checkbox(
                        im_str!("Entity List"),
                        &mut window_settings.show_entity_list,
                    );
                    ui.checkbox(im_str!("Inspector"), &mut window_settings.show_inspector);
                });
                ui.separator();
                ui.menu(im_str!("Debug Setings")).build(|| {
                    ui.checkbox(im_str!("Debug Window"), &mut debug_options.show_debug_info);
                });

                ui.separator();

                if is_edit_mode {
                    if ui.menu_item(im_str!("\u{e8c4} Reset")).build() {
                        game_control.enqueue_reset_level();
                        game_control.enqueue_change_play_mode(crate::PlayMode::System);
                    }

                    if ui.menu_item(im_str!("\u{f40a} Play")).build() {
                        game_control.enqueue_change_play_mode(crate::PlayMode::Playing);
                    }
                } else {
                    if ui.menu_item(im_str!("\u{e8c4} Reset")).build() {
                        game_control.enqueue_reset_level();
                        game_control.enqueue_change_play_mode(crate::PlayMode::System);
                    }

                    if ui.menu_item(im_str!("\u{f3e4} Pause")).build() {
                        game_control.enqueue_change_play_mode(crate::PlayMode::System);
                    }
                }

                ui.text(im_str!("FPS: {:.1}", time_state.system().fps_smoothed));
                ui.separator();
                ui.separator();
            });
        })
    }
}
