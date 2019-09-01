use crate::framework::FrameworkComponentPrototype;
use crate::components::EditorModifiedComponent;
use super::InspectorTab;
use hashbrown::HashMap;
use minimum::component::ComponentStorage;
use minimum::Component;
use minimum::EntityHandle;
use minimum::ResourceMap;
use std::marker::PhantomData;

use imgui_inspect::InspectArgsStruct;

enum InspectResult {
    Unchanged,
    Edited,
    Deleted
}

//
// Interface for a registered component type
//
trait RegisteredComponentTrait: Send + Sync {
    fn header_text(&self) -> &'static str;
    fn handled_type(&self) -> core::any::TypeId;

    fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui);
    fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) -> InspectResult;
}

pub struct RegisteredComponent<T>
where
    T: Component,
{
    header_text: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredComponent<T>
where
    T: Component,
{
    fn new(header_text: &'static str) -> Self {
        RegisteredComponent {
            header_text,
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredComponentTrait for RegisteredComponent<T>
where
    T: Component + imgui_inspect::InspectRenderStruct<T>,
{
    fn header_text(&self) -> &'static str {
        self.header_text
    }

    fn handled_type(&self) -> core::any::TypeId {
        core::any::TypeId::of::<T>()
    }

    fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui) {
        let storage = resource_map.fetch::<<T as Component>::Storage>();

        let mut data: Vec<&T> = vec![];
        //let mut all_entities_have_component = true;
        for entity_handle in entity_handles {
            let comp = storage.get(&entity_handle);
            if let Some(t) = comp {
                data.push(t);
            }
        }

        if data.len() > 0 {
            <T as imgui_inspect::InspectRenderStruct<T>>::render(
                data.as_slice(),
                core::any::type_name::<T>(),
                ui,
                &InspectArgsStruct::default(),
            );
        }
    }

    fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) -> InspectResult {
        let mut storage = resource_map.fetch_mut::<<T as Component>::Storage>();

        let mut data: Vec<&mut T> = vec![];
        //let mut all_entities_have_component = true;

        for entity_handle in entity_handles {
            let comp = storage.get_mut(&entity_handle);
            if let Some(t) = comp {
                // This unsafe block allows us to grab multiple mutible refs from the storage. It is
                // only safe if the storage does not change and we don't have duplicate entity handles
                unsafe {
                    let ptr: *mut T = t;
                    data.push(&mut *ptr);
                }
            }
        }

        let mut args = InspectArgsStruct::default();
        args.header = Some(false);
        args.indent_children = Some(false);

        if data.len() > 0 {
            let header_text = &imgui::im_str!("{}", self.header_text);
            let _content_region = ui.get_window_content_region_max();
            let draw_children = unsafe { imgui_sys::igCollapsingHeader(header_text.as_ptr(), imgui_sys::ImGuiTreeNodeFlags_DefaultOpen as i32 | imgui_sys::ImGuiTreeNodeFlags_AllowItemOverlap as i32) };
            if draw_children {

                //TODO: This is not woring well enough to be worth exposing
                /*
                ui.same_line(content_region[0] - 50.0);
                if ui.small_button(imgui::im_str!("Delete")) {
                    for e in entity_handles {
                        //TODO: This seems like it's drawing undefined values when something gets deleted...
                        //TODO: Deleting components isn't properly calling free handlers
                        storage.free(e);
                    }

                    // This component type was deleted on this frame
                    true
                } else
                */
                {

                    ui.indent();

                    let changed = <T as imgui_inspect::InspectRenderStruct<T>>::render_mut(
                        data.as_mut_slice(),
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
                }
            } else {
                // This component is collapsed, it cannot be edited
                InspectResult::Unchanged
            }
        } else {
            // This component type is not on the entity
            InspectResult::Unchanged
        }
    }
}

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn header_text(&self) -> &'static str;
    fn handled_type(&self) -> core::any::TypeId;

    fn render(
        &self,
        prototypes: &HashMap<core::any::TypeId, Vec<&Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    );
    fn render_mut(
        &self,
        prototypes: &mut HashMap<core::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    ) -> InspectResult;
}

pub struct RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderStruct<T>,
{
    header_text: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderStruct<T>,
{
    fn new(header_text: &'static str) -> Self {
        RegisteredComponentPrototype {
            header_text,
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderStruct<T> + named_type::NamedType,
{
    fn header_text(&self) -> &'static str {
        self.header_text
    }

    fn handled_type(&self) -> core::any::TypeId {
        core::any::TypeId::of::<T>()
    }

    fn render(
        &self,
        prototypes: &HashMap<std::any::TypeId, Vec<&Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    ) {
        if let Some(values) = prototypes.get(&std::any::TypeId::of::<T>()) {
            let mut cast_values: Vec<&T> = vec![];

            for v in values {
                cast_values.push(v.downcast_ref::<T>().unwrap());
            }

            <T as imgui_inspect::InspectRenderStruct<T>>::render(
                &cast_values,
                core::any::type_name::<T>(),
                ui,
                &InspectArgsStruct::default(),
            );
        }
    }

    fn render_mut(
        &self,
        prototypes: &mut HashMap<std::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    ) -> InspectResult
    {
        if let Some(values) = prototypes.get_mut(&std::any::TypeId::of::<T>()) {
            let mut cast_values: Vec<&mut T> = vec![];

            for v in values {
                cast_values.push(v.downcast_mut::<T>().unwrap());
            }

            let header_text = &imgui::im_str!("{}", self.header_text);
            let content_region = ui.get_window_content_region_max();
            ui.push_id(core::any::type_name::<T>());
            let draw_children = unsafe { imgui_sys::igCollapsingHeader(header_text.as_ptr(), imgui_sys::ImGuiTreeNodeFlags_DefaultOpen as i32 | imgui_sys::ImGuiTreeNodeFlags_AllowItemOverlap as i32) };
            ui.same_line(content_region[0] - 50.0);
            let result = if ui.small_button(imgui::im_str!("Delete")) {

                // The component was deleted
                InspectResult::Deleted

            } else if draw_children {
                ui.indent();

                let mut args = InspectArgsStruct::default();
                args.header = Some(false);
                args.indent_children = Some(false);

                let changed = <T as imgui_inspect::InspectRenderStruct<T>>::render_mut(
                    &mut cast_values,
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

            ui.pop_id();
            result
        } else {
            // This component type is not on the prototype
            InspectResult::Unchanged
        }
    }
}

//
// ComponentRegistry
//
pub struct InspectRegistry {
    registered_components: Vec<Box<dyn RegisteredComponentTrait>>,
    registered_component_prototypes: Vec<Box<dyn RegisteredComponentPrototypeTrait>>,
}

impl InspectRegistry {
    pub fn new() -> Self {
        InspectRegistry {
            registered_components: vec![],
            registered_component_prototypes: vec![],
        }
    }

    pub fn register_component<T: Component + 'static + imgui_inspect::InspectRenderStruct<T>>(
        &mut self,
        header_text: &'static str
    ) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T>::new(header_text)));
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototype + 'static + imgui_inspect::InspectRenderStruct<T>,
    >(
        &mut self,
        header_text: &'static str
    ) {
        self.registered_component_prototypes
            .push(Box::new(RegisteredComponentPrototype::<T>::new(header_text)));
    }

    pub fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
        set_inspector_tab: &mut Option<InspectorTab>
    ) {
        let tab_bar_str = imgui::im_str!("Inspector");

        unsafe {
            imgui_sys::igBeginTabBar(
                tab_bar_str.as_ptr(),
                imgui_sys::ImGuiTabBarFlags_None as i32,
            );
        }

        self.render_persistent_tab(resource_map, entity_handles, ui, set_inspector_tab);
        self.render_runtime_tab(resource_map, entity_handles, ui, set_inspector_tab);
        *set_inspector_tab = None;

        unsafe {
            imgui_sys::igEndTabBar();
        }
    }

    fn render_persistent_tab(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
        set_inspector_tab: &Option<InspectorTab>
    ) {

        // Gather all the prototype arcs we will be editing
        let mut storage = resource_map
            .fetch_mut::<<crate::components::PersistentEntityComponent as Component>::Storage>();

        {

            let persistent_tab_str = imgui::im_str!("Persistent");

            let mut tab_flags = imgui_sys::ImGuiTabItemFlags_None;
            if let Some(new_tab) = set_inspector_tab {
                if *new_tab == InspectorTab::Persistent {
                    tab_flags |= imgui_sys::ImGuiTabItemFlags_SetSelected;
                }
            }

            let tab_is_open;
            unsafe {
                tab_is_open = imgui_sys::igBeginTabItem(
                    persistent_tab_str.as_ptr(),
                    std::ptr::null_mut(),
                    tab_flags as i32,
                );
            }


            if tab_is_open {
                // Prototypes is going to hold mut refs to values within the arcs/locks, so be careful with lifetimes here. (See unsafe block below)
                let mut arcs = vec![];
                let mut locks = vec![];

                let mut prototypes =
                    HashMap::<core::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>::new();

                for entity_handle in entity_handles {
                    let comp = storage.get_mut(&entity_handle);
                    if let Some(comp) = comp {
                        let arc = comp.entity_prototype().inner().clone();
                        arcs.push(arc);
                    }
                }

                // Lock them all and put mut refs in the map
                for arc in &arcs {
                    let mut guard = arc.lock().unwrap();
                    let pep = &mut *guard;
                    for component_prototype in pep.component_prototypes_mut() {
                        let component_prototype_type =
                            FrameworkComponentPrototype::type_id(&**component_prototype);
                        let prototypes_entry =
                            prototypes.entry(component_prototype_type).or_insert(vec![]);

                        // This unsafe block allows us to grab multiple mutible refs from the storage. It is
                        // only safe if the storage does not change.
                        // As long as we're holding the WriteBorrow on PersistentEntityComponent storage
                        //prototypes_entry.push(&mut *component_prototype);
                        unsafe {
                            let component_prototype_ptr: *mut Box<dyn FrameworkComponentPrototype> =
                                component_prototype;
                            prototypes_entry.push(&mut *component_prototype_ptr);
                        }
                    }

                    locks.push(guard);
                }

                let mut component_types_to_delete = vec![];
                let mut mark_entity_modified = false;
                for rcp in &self.registered_component_prototypes {
                    match rcp.render_mut(&mut prototypes, ui) {
                        InspectResult::Edited => {
                            mark_entity_modified = true;
                        },
                        InspectResult::Deleted => {
                            component_types_to_delete.push(rcp.handled_type());
                        },
                        InspectResult::Unchanged => {}
                    }
                }

                unsafe {
                    imgui_sys::igEndTabItem();
                }

                if component_types_to_delete.len() > 0 {
                    for mut entity_prototype in locks {
                        // For each component prototype
                        let component_prototypes = entity_prototype.component_prototypes_mut();

                        // Iterate through the prototypes backwards so we can swap_remove them
                        for i in (0..component_prototypes.len()).rev() {
                            let component_prototype = &component_prototypes[i];
                            let type_id = FrameworkComponentPrototype::type_id(&**component_prototype);

                            for component_type_to_delete in &component_types_to_delete {
                                if type_id == *component_type_to_delete {
                                    component_prototypes.swap_remove(i);
                                    mark_entity_modified = true;
                                }
                            }
                        }
                    }
                }

                // Put a EditorModifiedComponent component on all the given entities
                if mark_entity_modified {
                    let mut editor_modified_components = resource_map.fetch_mut::<<EditorModifiedComponent as Component>::Storage>();
                    for entity_handle in entity_handles {
                        if !editor_modified_components.exists(&entity_handle) {
                            editor_modified_components.allocate(&entity_handle, EditorModifiedComponent::new());
                        }
                    }
                }
            }
        };
    }

    fn render_runtime_tab(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
        set_inspector_tab: &Option<InspectorTab>
    ) {
        let runtime_tab_str = imgui::im_str!("Runtime");

        let mut tab_flags = imgui_sys::ImGuiTabItemFlags_None;
        if let Some(new_tab) = set_inspector_tab {
            if *new_tab == InspectorTab::Runtime {
                tab_flags |= imgui_sys::ImGuiTabItemFlags_SetSelected;
            }
        }

        let tab_is_open;
        unsafe {
            tab_is_open = imgui_sys::igBeginTabItem(
                runtime_tab_str.as_ptr(),
                std::ptr::null_mut(),
                tab_flags as i32,
            );
        }

        if tab_is_open {
            for rc in &self.registered_components {
                match rc.render_mut(resource_map, entity_handles, ui) {
                    InspectResult::Edited => {},
                    InspectResult::Deleted => {
                        //TODO: Implement
                    },
                    InspectResult::Unchanged => {},
                }
            }

            unsafe {
                imgui_sys::igEndTabItem();
            }
        }
    }
}
