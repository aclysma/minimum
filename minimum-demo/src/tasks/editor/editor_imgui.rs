use minimum::resource::{DataRequirement, Read, Write, WriteBorrow};
use minimum::{Task, TaskContext, ReadComponent};

use crate::resources::{DebugDraw, EditorCollisionWorld, InputManager, RenderState, ImguiManager};

use crate::components::EditorSelectedComponent;
use named_type::NamedType;
use crate::inspect::{InspectRenderDefault, MyStruct2Optionized};

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
//unsafe {
//    imgui::sys::igListBoxHeaderInt(im_str!("entities").as_ptr(), 5, 20);
//    imgui::sys::igSelectable(im_str!("a").as_ptr(), true, imgui::sys::ImGuiSelectableFlags_None as i32, imgui::sys::ImVec2::new(20.0, 20.0));
//    imgui::sys::igListBoxFooter();
//
//}


                let s2_internal = crate::inspect::MyStruct { a: 1.0, b: 2.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0) };
                let mut s2 = crate::inspect::MyStruct2 { a: 1.0, b: 2.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0), s: s2_internal };

                let s3_internal = crate::inspect::MyStruct { a: 1.0, b: 2.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0) };
                let mut s3 = crate::inspect::MyStruct2 { a: 1.0, b: 2.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0), s: s3_internal };
                let s4_internal = crate::inspect::MyStruct { a: 1.0, b: 2.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0) };
                let mut s4 = crate::inspect::MyStruct2 { a: 1.0, b: 3.0, c: glm::vec2(2.5, 4.3), d: glm::vec3(100.0, 200.0, 300.0), s: s4_internal };

                let mut optionized = minimum::util::optionize::create_optionized_from_set::<MyStruct2Optionized, _>(&[&s3, &s4]);

                optionized.render_mut("var", ui);

            })
        })
    }
}
