use crate::framework::persist::ComponentPrototypeSerializer;
use crate::framework::FrameworkComponentPrototype;
use minimum::Component;
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::ResourceMap;
use named_type::NamedType;

use imgui_inspect::InspectRenderDefault;

use imgui_inspect::InspectArgsDefault;
use minimum::component::ComponentCreateQueueFlushListener;
use serde::de::DeserializeOwned;
use serde::Serialize;

// The only reason for wrapping BasicComponentPrototype and BasicComponentFactory is so that traits
// can be added non-intrusively

#[derive(Clone, NamedType)]
pub struct CloneComponentPrototype<T: Component + Clone> {
    inner: minimum::BasicComponentPrototype<T>,
}

impl<T: Component + Clone> CloneComponentPrototype<T> {
    pub fn new(clone: T) -> Self {
        CloneComponentPrototype::<T> {
            inner: minimum::BasicComponentPrototype::new(clone),
        }
    }

    pub fn data(&self) -> &T {
        self.inner.data()
    }

    pub fn data_mut(&mut self) -> &mut T {
        self.inner.data_mut()
    }
}

impl<T: Component + Clone> ComponentPrototype for CloneComponentPrototype<T> {
    type Factory = CloneComponentFactory<T>;
}

impl<T: Component + Clone + InspectRenderDefault<T>> FrameworkComponentPrototype
    for CloneComponentPrototype<T>
{
}

//
// Implement inspect for clone components
//
impl<T> imgui_inspect::InspectRenderDefault<CloneComponentPrototype<T>>
    for CloneComponentPrototype<T>
where
    T: Component + Clone + InspectRenderDefault<T> + named_type::NamedType,
{
    fn render(
        data: &[&CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &imgui_inspect::InspectArgsDefault,
    ) {
        let mut r = vec![];
        for d in data {
            let v = d.data();
            r.push(v);
        }

        <T as imgui_inspect::InspectRenderDefault<T>>::render(r.as_slice(), label, ui, args)
    }

    fn render_mut(
        data: &mut [&mut CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut r = vec![];
        for d in data {
            let v = d.data_mut();
            r.push(v);
        }

        <T as imgui_inspect::InspectRenderDefault<T>>::render_mut(r.as_mut_slice(), label, ui, args)
    }
}

impl<T: Component + Clone + Serialize + DeserializeOwned>
    ComponentPrototypeSerializer<CloneComponentPrototype<T>> for CloneComponentPrototype<T>
{
    fn serialize(prototype: &CloneComponentPrototype<T>) -> Result<String, failure::Error> {
        Ok(serde_json::to_string(prototype.inner.data())?)
    }

    fn deserialize(data: &str) -> Result<CloneComponentPrototype<T>, failure::Error> {
        Ok(CloneComponentPrototype::<T>::new(serde_json::from_str(
            &data,
        )?))
    }
}

pub struct CloneComponentFactory<T: Component> {
    inner: minimum::BasicComponentFactory<T>,
}

impl<T: Component> CloneComponentFactory<T> {
    pub fn new() -> Self {
        CloneComponentFactory::<T> {
            inner: minimum::BasicComponentFactory::new(),
        }
    }
}

impl<T: Component + Clone> ComponentFactory<CloneComponentPrototype<T>>
    for CloneComponentFactory<T>
{
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &CloneComponentPrototype<T>,
    ) {
        self.inner.enqueue_create(entity_handle, &prototype.inner);
    }
}

impl<T: Component + Clone> ComponentCreateQueueFlushListener for CloneComponentFactory<T> {
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        self.inner.flush_creates(resource_map, entity_set);
    }
}
