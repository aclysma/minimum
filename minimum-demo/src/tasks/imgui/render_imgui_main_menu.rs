use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, DispatchControl, EntitySet, ReadComponent, Task, TaskContext};

use crate::resources::{
    EditorUiState, GameControl, DebugOptions, ImguiManager, InputManager, PhysicsManager, RenderState, TimeState,
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
        Write<EditorUiState>,
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
            mut editor_ui_state,
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
                    ui.checkbox(
                        im_str!("ImGui Metrics"),
                        &mut window_settings.show_imgui_metrics,
                    );
                    ui.checkbox(
                        im_str!("ImGui Style Editor"),
                        &mut window_settings.show_imgui_style_editor,
                    );
                    ui.checkbox(
                        im_str!("Entity List"),
                        &mut window_settings.show_entity_list,
                    );
                });
                ui.separator();
                ui.menu(im_str!("Debug Setings")).build(|| {
                    ui.checkbox(im_str!("Debug Window"), &mut debug_options.show_debug_info);
                });

                ui.separator();

                if is_edit_mode {
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
        })
    }
}
