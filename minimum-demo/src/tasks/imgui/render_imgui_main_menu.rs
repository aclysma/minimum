use minimum::resource::{DataRequirement, Read, Write};
use minimum::{Task, TaskContext};

use framework::resources::{EditorUiState, EditorTool, FrameworkActionQueue, TimeState};
use crate::resources::{DebugOptions, ImguiManager};

use named_type::NamedType;
use imgui::im_str;

#[derive(NamedType)]
pub struct RenderImguiMainMenu;

impl RenderImguiMainMenu {
    fn tool_button(ui: &imgui::Ui, editor_ui_state: &mut EditorUiState, editor_tool: EditorTool, string: &'static str) {
        let _token = if editor_ui_state.active_editor_tool == editor_tool {
            Some(ui.push_style_color(imgui::StyleColor::Text, [0.8, 0.0, 0.0, 1.0]))
        } else {
            None
        };

        if ui.menu_item(&im_str!("{}", string)).build() {
            editor_ui_state.active_editor_tool = editor_tool;
        }
    }
}

impl Task for RenderImguiMainMenu {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Write<FrameworkActionQueue>,
        Write<EditorUiState>,
        Write<DebugOptions>,
    );
    const REQUIRED_FLAGS: usize = framework::context_flags::AUTHORITY_CLIENT as usize;

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

        let is_edit_mode = time_state.play_mode == framework::PlayMode::System;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            {
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
            }

            ui.main_menu_bar(|| {

                //cursor-default-outline
                Self::tool_button(ui, &mut *editor_ui_state, EditorTool::Select, "\u{f1b5}");
                //axis-arrow
                Self::tool_button(ui, &mut *editor_ui_state, EditorTool::Translate, "\u{fd25}");
                //rotate-orbit
                Self::tool_button(ui, &mut *editor_ui_state, EditorTool::Rotate, "\u{fd74}");
                //resize
                Self::tool_button(ui, &mut *editor_ui_state, EditorTool::Scale, "\u{fa67}");

                ui.menu(im_str!("File")).build(|| {
                    ui.menu(im_str!("Sub Menu")).build(|| {
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

                    if ui.menu_item(im_str!("New")).build() {
                        game_control.enqueue_new_level();
                    }

                    if ui.menu_item(im_str!("Load")).build() {
                        game_control.enqueue_load_level(std::path::PathBuf::from("test_save"));
                    }

                    if ui.menu_item(im_str!("Save")).build() {
                        game_control.enqueue_save_level(std::path::PathBuf::from("test_save"));
                    }
                });

                let window_settings = editor_ui_state.window_options_mut(time_state.play_mode);
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
                        game_control.enqueue_change_play_mode(framework::PlayMode::System);
                    }

                    if ui.menu_item(im_str!("\u{f40a} Play")).build() {
                        game_control.enqueue_change_play_mode(framework::PlayMode::Playing);
                    }
                } else {
                    if ui.menu_item(im_str!("\u{e8c4} Reset")).build() {
                        game_control.enqueue_reset_level();
                        game_control.enqueue_change_play_mode(framework::PlayMode::System);
                    }

                    if ui.menu_item(im_str!("\u{f3e4} Pause")).build() {
                        game_control.enqueue_change_play_mode(framework::PlayMode::System);
                    }
                }

                ui.text(im_str!("FPS: {:.1}", time_state.system().fps_smoothed));
                ui.separator();
                ui.separator();
            });
        })
    }
}
