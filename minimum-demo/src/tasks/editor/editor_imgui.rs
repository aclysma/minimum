use minimum::resource::{DataRequirement, Read, Write, WriteBorrow};
use minimum::ComponentStorage;
use minimum::{EntityHandle, Task, TaskContext, WriteComponent};

use crate::resources::{DebugDraw, EditorCollisionWorld, InputManager, MouseButtons, RenderState, ImguiManager};

use crate::components::EditorSelectedComponent;
use crate::tasks::DebugDrawComponents;
use ncollide2d::world::CollisionGroups;

use winit::event::VirtualKeyCode;

#[derive(typename::TypeName)]
pub struct EditorImgui;
impl Task for EditorImgui {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<RenderState>,
        Read<EditorCollisionWorld>,
        WriteComponent<EditorSelectedComponent>,
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
            input_manager,
            render_state,
            editor_collision_world,
            mut editor_selected_components,
            mut debug_draw,
            mut imgui_manager
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
            use imgui::im_str;
            ui.window(im_str!("Editor")).build((|| {
                ui.text(format!("Selected Entities: {}", editor_selected_components.count()));
//unsafe {
//    imgui::sys::igListBoxHeaderInt(im_str!("entities").as_ptr(), 5, 20);
//    imgui::sys::igSelectable(im_str!("a").as_ptr(), true, imgui::sys::ImGuiSelectableFlags_None as i32, imgui::sys::ImVec2::new(20.0, 20.0));
//    imgui::sys::igListBoxFooter();
//
//}
            }))
        })
    }
}
