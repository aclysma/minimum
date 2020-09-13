use legion::*;

use minimum_game::resources::TimeResource;
use crate::resources::{EditorStateResource};
use minimum_game::resources::ImguiResource;
use crate::resources::EditorTool;

use imgui::im_str;

fn imgui_menu_tool_button(
    ui: &imgui::Ui,
    editor_state: &mut EditorStateResource,
    editor_tool: EditorTool,
    string: &'static str,
) {
    let color_stack_token = if editor_state.active_editor_tool() == editor_tool {
        Some(ui.push_style_color(imgui::StyleColor::Text, [0.8, 0.0, 0.0, 1.0]))
    } else {
        None
    };

    if imgui::MenuItem::new(&im_str!("{}", string)).build(ui) {
        editor_state.enqueue_set_active_editor_tool(editor_tool);
    }

    if let Some(color_stack_token) = color_stack_token {
        color_stack_token.pop(ui);
    }
}

pub fn editor_imgui_menu(schedule: &mut legion::systems::Builder) {
    schedule.add_system(
        SystemBuilder::new("editor_imgui_menu")
            .write_resource::<ImguiResource>()
            .write_resource::<EditorStateResource>()
            .read_resource::<TimeResource>()
            .build(|_command_buffer, _, (imgui, editor_state, time_state), _| {
                imgui.with_ui(|ui| {
                    {
                        let window_settings = editor_state.window_options_mut();
                        if window_settings.show_imgui_metrics {
                            ui.show_metrics_window(&mut window_settings.show_imgui_metrics);
                        }

                        if window_settings.show_imgui_style_editor {
                            imgui::Window::new(im_str!("Editor")).build(ui, || {
                                ui.show_default_style_editor();
                            });
                        }

                        if window_settings.show_imgui_demo {
                            ui.show_demo_window(&mut window_settings.show_imgui_demo);
                        }
                    }

                    ui.main_menu_bar(|| {
                        //axis-arrow
                        imgui_menu_tool_button(
                            ui,
                            &mut *editor_state,
                            EditorTool::Translate,
                            "\u{fd25}",
                        );
                        //resize
                        imgui_menu_tool_button(
                            ui,
                            &mut *editor_state,
                            EditorTool::Scale,
                            "\u{fa67}",
                        );
                        //rotate-orbit
                        imgui_menu_tool_button(
                            ui,
                            &mut *editor_state,
                            EditorTool::Rotate,
                            "\u{fd74}",
                        );

                        ui.menu(imgui::im_str!("File"), true, || {
                            if imgui::MenuItem::new(imgui::im_str!("Open")).build(ui) {
                                if let Some(opened_prefab) = editor_state.opened_prefab() {
                                    // temporarily get the recently opened prefab uuid from editor state
                                    let uuid = opened_prefab.uuid();
                                    editor_state.enqueue_open_prefab(*uuid);
                                }
                            }

                            if imgui::MenuItem::new(im_str!("Save")).build(ui) {
                                editor_state.enqueue_save_prefab();
                            }
                        });

                        ui.menu(imgui::im_str!("Edit"), true, || {
                            if imgui::MenuItem::new(im_str!("Undo")).build(ui) {
                                editor_state.enqueue_undo();
                            }

                            if imgui::MenuItem::new(im_str!("Redo")).build(ui) {
                                editor_state.enqueue_redo();
                            }
                        });

                        let window_settings = editor_state.window_options_mut();
                        ui.menu(im_str!("Windows"), true, || {
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

                        if editor_state.is_editor_active() {
                            if imgui::MenuItem::new(im_str!("\u{e8c4} Reset")).build(ui) {
                                editor_state.enqueue_reset();
                            }

                            if imgui::MenuItem::new(im_str!("\u{f40a} Play")).build(ui) {
                                editor_state.enqueue_play();
                            }
                        } else {
                            if imgui::MenuItem::new(im_str!("\u{e8c4} Reset")).build(ui) {
                                editor_state.enqueue_reset();
                            }

                            if imgui::MenuItem::new(im_str!("\u{f3e4} Pause")).build(ui) {
                                editor_state.enqueue_pause();
                            }
                        }

                        ui.text(im_str!(
                            "FPS: {:.1}",
                            time_state.system_time().updates_per_second_smoothed()
                        ));

                        if time_state.is_simulation_paused() {
                            ui.text(im_str!("SIMULATION PAUSED"));
                        }
                    });
                });
            }),
    );
}
