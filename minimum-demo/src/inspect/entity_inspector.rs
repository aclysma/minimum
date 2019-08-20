
use std::marker::PhantomData;
use minimum::ResourceMap;
use minimum::EntityHandle;
use minimum::Component;
use minimum::component::ComponentRegistry;

//
// Interface for a registered component type
//
trait RegisteredInspectorComponentTrait: Send + Sync {
    fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui);
    fn render_mut(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui);
}

pub struct RegisteredInspectorComponent<T>
    where
        T: Component
{
    phantom_data: PhantomData<T>
}

impl<T> RegisteredInspectorComponent<T>
    where
        T: Component
{
    fn new() -> Self {
        RegisteredInspectorComponent {
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredInspectorComponentTrait for RegisteredInspectorComponent<T>
    where
        T: Component + imgui_inspect::InspectRenderDefault<T>
{
    fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui) {
        let storage = resource_map.fetch::<<T as Component>::Storage>();

        use minimum::component::ComponentStorage;

        let mut data : Vec<&T> = vec![];
        let mut all_entities_have_component = true;
        for entity_handle in entity_handles {
            let comp = storage.get(&entity_handle);
            if let Some(t) = comp {
                data.push(t);
            }
        }

        if data.len() > 0 {
            <T as imgui_inspect::InspectRenderDefault<T>>::render(data.as_slice(), "label", ui, &imgui_inspect::InspectArgsDefault::default());
        }
    }

    fn render_mut(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui) {
        let mut storage = resource_map.fetch_mut::<<T as Component>::Storage>();

        use minimum::component::ComponentStorage;

        let mut data : Vec<&mut T> = vec![];
        let mut all_entities_have_component = true;

        for entity_handle in entity_handles {
            let comp = storage.get_mut(&entity_handle);
            if let Some(t) = comp {
                // This unsafe block allows us to grab multiple mutible refs from the storage. It is
                // only safe if the storage does not change and we don't have duplicate entity handles
                unsafe {
                    let ptr : *mut T = t;
                    data.push(&mut *ptr);
                }
            }
        }

        if data.len() > 0 {
            <T as imgui_inspect::InspectRenderDefault<T>>::render_mut(data.as_mut_slice(), "label", ui, &imgui_inspect::InspectArgsDefault::default());
        }
    }
}

//
// ComponentRegistry
//
pub struct InspectorComponentRegistry {
    registered_components: Vec<Box<dyn RegisteredInspectorComponentTrait>>
}

impl InspectorComponentRegistry {
    pub fn new() -> Self {
        InspectorComponentRegistry {
            registered_components: vec![]
        }
    }

    pub fn register_component<T: Component + 'static + imgui_inspect::InspectRenderDefault<T>>(&mut self) {
        self.registered_components
            .push(Box::new(RegisteredInspectorComponent::<T>::new()));
    }

    pub fn render(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui) {
        for rc in &self.registered_components {
            rc.render(resource_map, entity_handles, ui);
        }
    }

    pub fn render_mut(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle], ui: &imgui::Ui) {
        for rc in &self.registered_components {
            rc.render_mut(resource_map, entity_handles, ui);
        }
    }
}