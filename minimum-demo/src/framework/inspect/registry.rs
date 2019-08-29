use crate::framework::FrameworkComponentPrototype;
use hashbrown::HashMap;
use minimum::component::ComponentStorage;
use minimum::Component;
use minimum::EntityHandle;
use minimum::ResourceMap;
use std::marker::PhantomData;

use imgui_inspect::InspectArgsDefault;

//
// Interface for a registered component type
//
trait RegisteredComponentTrait: Send + Sync {
    fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui);
    fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    );
}

pub struct RegisteredComponent<T>
where
    T: Component,
{
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredComponent<T>
where
    T: Component,
{
    fn new() -> Self {
        RegisteredComponent {
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredComponentTrait for RegisteredComponent<T>
where
    T: Component + imgui_inspect::InspectRenderDefault<T>,
{
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
            <T as imgui_inspect::InspectRenderDefault<T>>::render(
                data.as_slice(),
                "label",
                ui,
                &InspectArgsDefault::default(),
            );
        }
    }

    fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) {
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

        if data.len() > 0 {
            <T as imgui_inspect::InspectRenderDefault<T>>::render_mut(
                data.as_mut_slice(),
                "label",
                ui,
                &InspectArgsDefault::default(),
            );
        }
    }
}

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn render(
        &self,
        prototypes: &HashMap<core::any::TypeId, Vec<&Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    );
    fn render_mut(
        &self,
        prototypes: &mut HashMap<core::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    );
}

pub struct RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderDefault<T>,
{
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderDefault<T>,
{
    fn new() -> Self {
        RegisteredComponentPrototype {
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + imgui_inspect::InspectRenderDefault<T> + named_type::NamedType,
{
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

            <T as imgui_inspect::InspectRenderDefault<T>>::render(
                &cast_values,
                T::type_name(),
                ui,
                &InspectArgsDefault::default(),
            );
        }
    }

    fn render_mut(
        &self,
        prototypes: &mut HashMap<std::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>,
        ui: &imgui::Ui,
    ) {
        if let Some(values) = prototypes.get_mut(&std::any::TypeId::of::<T>()) {
            let mut cast_values: Vec<&mut T> = vec![];

            for v in values {
                cast_values.push(v.downcast_mut::<T>().unwrap());
            }

            <T as imgui_inspect::InspectRenderDefault<T>>::render_mut(
                &mut cast_values,
                T::type_name(),
                ui,
                &InspectArgsDefault::default(),
            );
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

    pub fn register_component<T: Component + 'static + imgui_inspect::InspectRenderDefault<T>>(
        &mut self,
    ) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T>::new()));
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototype + 'static + imgui_inspect::InspectRenderDefault<T>,
    >(
        &mut self,
    ) {
        self.registered_component_prototypes
            .push(Box::new(RegisteredComponentPrototype::<T>::new()));
    }

    pub fn render(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) {
        for rc in &self.registered_components {
            rc.render(resource_map, entity_handles, ui);
        }
    }

    pub fn render_mut(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) {
        let tab_bar_str = imgui::im_str!("Inspector");

        unsafe {
            imgui_sys::igBeginTabBar(
                tab_bar_str.as_ptr(),
                imgui_sys::ImGuiTabBarFlags_None as i32,
            );
        }

        self.render_persistent_tab(resource_map, entity_handles, ui);
        self.render_runtime_tab(resource_map, entity_handles, ui);

        unsafe {
            //imgui_sys::igEndTabItem();
            imgui_sys::igEndTabBar();
        }
    }

    fn render_persistent_tab(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) {
        let persistent_tab_str = imgui::im_str!("Persistent");

        let tab_is_open;
        unsafe {
            tab_is_open = imgui_sys::igBeginTabItem(
                persistent_tab_str.as_ptr(),
                std::ptr::null_mut(),
                imgui_sys::ImGuiTabItemFlags_None as i32,
            );
        }

        if tab_is_open {
            // Prototypes is going to hold mut refs to values within the arcs/locks, so be careful with lifetimes here. (See unsafe block below)
            let mut arcs = vec![];
            let mut locks = vec![];

            let mut prototypes =
                HashMap::<core::any::TypeId, Vec<&mut Box<dyn FrameworkComponentPrototype>>>::new();

            // Gather all the prototype arcs we will be editing
            let mut storage = resource_map
                .fetch_mut::<<crate::components::PersistentEntityComponent as Component>::Storage>(
            );
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

            for rcp in &self.registered_component_prototypes {
                rcp.render_mut(&mut prototypes, ui);
            }

            unsafe {
                imgui_sys::igEndTabItem();
            }
        }
    }

    fn render_runtime_tab(
        &self,
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        ui: &imgui::Ui,
    ) {
        let runtime_tab_str = imgui::im_str!("Runtime");

        let tab_is_open;
        unsafe {
            tab_is_open = imgui_sys::igBeginTabItem(
                runtime_tab_str.as_ptr(),
                std::ptr::null_mut(),
                imgui_sys::ImGuiTabItemFlags_None as i32,
            );
        }

        if tab_is_open {
            for rc in &self.registered_components {
                rc.render_mut(resource_map, entity_handles, ui);
            }

            unsafe {
                imgui_sys::igEndTabItem();
            }
        }
    }
}
