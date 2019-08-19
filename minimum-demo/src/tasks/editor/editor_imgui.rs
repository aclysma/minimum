use minimum::resource::{DataRequirement, Read, Write, WriteBorrow};
use minimum::{Task, TaskContext, ReadComponent};

use crate::resources::{DebugDraw, EditorCollisionWorld, InputManager, RenderState, ImguiManager};

use crate::components::EditorSelectedComponent;
use named_type::NamedType;

#[derive(NamedType)]
pub struct EditorImgui;
impl Task for EditorImgui {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<RenderState>,
        Read<EditorCollisionWorld>,
        ReadComponent<EditorSelectedComponent>,
        Write<DebugDraw>,
        Write<ImguiManager>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_SYSTEM;

    fn run(
        &mut self,
        task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            _entity_set,
            _input_manager,
            _render_state,
            _editor_collision_world,
            editor_selected_components,
            mut _debug_draw,
            imgui_manager
        ) = data;

        if task_context.context_flags()
            & (crate::context_flags::PLAYMODE_PAUSED | crate::context_flags::PLAYMODE_PLAYING)
            != 0
        {
            return;
        }

        use imgui::im_str;

        let mut imgui_manager : WriteBorrow<ImguiManager> = imgui_manager;
        imgui_manager.with_ui(|ui| {
            ui.window(im_str!("Editor")).build(|| {
                ui.text(format!("Selected Entities: {}", editor_selected_components.count()));
            })
        });
    }
}
