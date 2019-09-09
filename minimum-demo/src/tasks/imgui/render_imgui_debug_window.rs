/*

use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, DispatchControl, EntitySet, ReadComponent, Task, TaskContext};

use crate::resources::{
    EditorUiState, GameControl, DebugOptions, ImguiManager, InputManager, PhysicsManager, RenderState, TimeState,
};

use crate::components::{BulletComponent, PlayerComponent, PositionComponent};


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
        ) = data;

        let is_edit_mode = time_state.play_mode == crate::PlayMode::System;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;


            if debug_options.show_debug_info {
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
*/