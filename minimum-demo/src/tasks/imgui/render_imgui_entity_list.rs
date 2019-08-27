use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, DispatchControl, EntitySet, ReadComponent, Task, TaskContext, WriteComponent};

use crate::resources::{
    DebugOptions, GameControl, ImguiManager, InputManager, PhysicsManager, RenderState, TimeState,
};

use crate::components::{BulletComponent, PlayerComponent, PositionComponent, EditorSelectedComponent};
use named_type::NamedType;

#[derive(NamedType)]
pub struct RenderImguiEntityList;
impl Task for RenderImguiEntityList {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Write<DebugOptions>,
        Read<EntitySet>,
        WriteComponent<EditorSelectedComponent>,
        Read<InputManager>
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
            mut debug_options,
            entity_set,
            mut editor_selected_components,
            input_manager
        ) = data;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

            use imgui::ImGuiSelectableFlags;

            if debug_options.show_entity_list {
                ui.window(im_str!("Entity List"))
                    .position([0.0, 50.0], imgui::Condition::Once)
                    .size([200.0, 200.0], imgui::Condition::Once)
                    .build(|| {
//                    ui.button(im_str!("btn"), [50.0, 0.0]);
//                    ui.same_line_with_spacing(50.0, 10.0);
//                    ui.button(im_str!("btn"), [50.0, 0.0]);

                    let name = im_str!("");
                    if unsafe { imgui_sys::igListBoxHeaderVec2(name.as_ptr(), imgui_sys::ImVec2 {x: -1.0, y: -1.0}) } {
                        for entity in entity_set.iter() {
                            let is_selected = if let Some(selected_component) = editor_selected_components.get(&entity.handle()) {
                                true
                            } else {
                                false
                            };

                            let s = im_str!("{:?}", entity.handle());
                            let clicked = ui.selectable(&s, is_selected, ImGuiSelectableFlags::empty(), [0.0, 0.0]);

                            if clicked {

                                let is_control_held = input_manager.is_key_down(winit::event::VirtualKeyCode::LControl);
                                if is_control_held {
                                    if !editor_selected_components.exists(&entity.handle()) {
                                        editor_selected_components.allocate(&entity.handle(), EditorSelectedComponent::new());
                                    } else {
                                        editor_selected_components.free(&entity.handle());
                                    }
                                } else {
                                    editor_selected_components.free_all();
                                    if !editor_selected_components.exists(&entity.handle()) {
                                        editor_selected_components.allocate(&entity.handle(), EditorSelectedComponent::new());
                                    }
                                }
                            }
                        }

                        unsafe {
                            imgui_sys::igListBoxFooter();
                        }
                    }
                });
            }
        })
    }
}
