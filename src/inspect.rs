use legion::prelude::*;

use std::marker::PhantomData;

use imgui::sys as imgui_sys;
use imgui::Ui;
use imgui_inspect::InspectRenderStruct;
use imgui_inspect::InspectArgsStruct;

#[derive(PartialEq)]
enum InspectResult {
    Unchanged,
    Edited,
    Deleted,
}

/// A trait object which allows dynamic dispatch into the selection implementation
trait RegisteredEditorInspectorT: Send + Sync {
    fn render(
        &self,
        world: &World,
        ui: &Ui,
        args: &InspectArgsStruct,
    );

    fn render_mut(
        &self,
        world: &mut World,
        entities: &[Entity],
        ui: &Ui,
        args: &InspectArgsStruct,
    ) -> InspectResult;
}

/// Implements the RegisteredEditorSelectableT trait object with code that can call
/// create_editor_selection_world on T
#[derive(Default)]
struct RegisteredEditorInspector<T> {
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredEditorInspector<T>
where
    T: InspectRenderStruct<T>,
{
    fn new() -> Self {
        RegisteredEditorInspector {
            phantom_data: Default::default(),
        }
    }
}

impl<T> RegisteredEditorInspectorT for RegisteredEditorInspector<T>
where
    T: InspectRenderStruct<T> + legion::storage::Component,
{
    fn render(
        &self,
        world: &World,
        ui: &Ui,
        args: &InspectArgsStruct,
    ) {
        let values = world.get_all_components::<T>();
        let slice = values.as_slice();

        if !slice.is_empty() {
            <T as InspectRenderStruct<T>>::render(slice, core::any::type_name::<T>(), ui, args);
        }
    }

    fn render_mut(
        &self,
        world: &mut World,
        entities: &[Entity],
        ui: &Ui,
        args: &InspectArgsStruct,
    ) -> InspectResult {
        let result = {
            let mut values = world.get_all_components_mut::<T>();
            let mut slice = values.as_mut_slice();

            if !slice.is_empty() {
                let header_text = &imgui::im_str!("{}", core::any::type_name::<T>());
                let content_region = ui.window_content_region_max();

                let id_token = ui.push_id(core::any::type_name::<T>());
                let draw_children = unsafe {
                    imgui_sys::igCollapsingHeader(
                        header_text.as_ptr(),
                        imgui_sys::ImGuiTreeNodeFlags_DefaultOpen as i32
                            | imgui_sys::ImGuiTreeNodeFlags_AllowItemOverlap as i32,
                    )
                };

                ui.same_line(content_region[0] - 50.0);

                let result = if ui.small_button(imgui::im_str!("Delete")) {
                    InspectResult::Deleted
                } else if draw_children {
                    ui.indent();

                    let mut args = InspectArgsStruct::default();
                    args.header = Some(false);
                    args.indent_children = Some(false);

                    let changed = <T as InspectRenderStruct<T>>::render_mut(
                        slice,
                        core::any::type_name::<T>(),
                        ui,
                        &args,
                    );

                    ui.unindent();

                    // This component is expanded, return if any fields were changed
                    if changed {
                        InspectResult::Edited
                    } else {
                        InspectResult::Unchanged
                    }
                } else {
                    // This component is collapsed, it cannot be edited
                    InspectResult::Unchanged
                };

                id_token.pop(ui);
                result
            } else {
                // This component type is not on the selected entities
                InspectResult::Unchanged
            }
        };

        if result == InspectResult::Deleted {
            for e in entities {
                world.remove_component::<T>(*e);
            }
        }

        result
    }
}

#[derive(Default)]
pub struct EditorInspectRegistry {
    registered: Vec<Box<dyn RegisteredEditorInspectorT>>,
}

impl EditorInspectRegistry {
    /// Adds a type to the registry, which allows components of these types to receive a callback
    /// to insert shapes into the collision world used for selection
    pub fn register<T: InspectRenderStruct<T> + legion::storage::Component>(&mut self) {
        self.registered
            .push(Box::new(RegisteredEditorInspector::<T>::new()));
    }

    pub fn render(
        &self,
        world: &World,
        ui: &Ui,
        args: &InspectArgsStruct,
    ) {
        for r in &self.registered {
            r.render(world, ui, args);
        }
    }

    pub fn render_mut(
        &self,
        world: &mut World,
        entities: &[Entity],
        ui: &Ui,
        args: &InspectArgsStruct,
    ) -> bool {
        let mut changed = false;
        for r in &self.registered {
            let result = r.render_mut(world, entities, ui, args);

            changed |= match result {
                InspectResult::Unchanged => false,
                _ => true,
            }
        }

        changed
    }
}
