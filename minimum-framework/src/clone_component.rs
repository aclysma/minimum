use crate::persist::ComponentPrototypeSerializer;
#[cfg(feature = "editor")]
use crate::select::SelectableComponentPrototype;
use minimum::BasicComponentPrototype;
use minimum::Component;
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::ResourceMap;

use minimum::component::ComponentCreateQueueFlushListener;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[cfg(feature = "editor")]
use imgui_inspect::{InspectArgsDefault, InspectArgsStruct, InspectRenderStruct};
use crate::prototype::FrameworkComponentPrototype;

// The only reason for wrapping BasicComponentPrototype and BasicComponentFactory is so that traits
// can be added non-intrusively

#[derive(Clone)]
pub struct CloneComponentPrototype<T: Component + Clone> {
    inner: minimum::BasicComponentPrototype<T>,
}

impl<T: Component + Clone> CloneComponentPrototype<T> {
    pub fn new(clone: T) -> Self {
        CloneComponentPrototype::<T> {
            inner: BasicComponentPrototype::new(clone),
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

impl<T: Component + Clone> FrameworkComponentPrototype for CloneComponentPrototype<T> {
    fn component_type() -> std::any::TypeId { std::any::TypeId::of::<T>() }
}

impl<T: Component + Clone + Default> Default for CloneComponentPrototype<T> {
    fn default() -> Self {
        CloneComponentPrototype {
            inner: BasicComponentPrototype::new(Default::default()),
        }
    }
}

//
// Implement inspect for clone components
//
#[cfg(feature = "editor")]
impl<T> imgui_inspect::InspectRenderDefault<CloneComponentPrototype<T>>
    for CloneComponentPrototype<T>
where
    T: Component + Clone + InspectRenderStruct<T>,
{
    fn render(
        data: &[&CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &imgui_inspect::InspectArgsDefault,
    ) {
        <Self as imgui_inspect::InspectRenderStruct<CloneComponentPrototype<T>>>::render(
            data,
            label,
            ui,
            &InspectArgsStruct::from((*args).clone()),
        )
    }

    fn render_mut(
        data: &mut [&mut CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        <Self as imgui_inspect::InspectRenderStruct<CloneComponentPrototype<T>>>::render_mut(
            data,
            label,
            ui,
            &InspectArgsStruct::from((*args).clone()),
        )
    }
}
#[cfg(feature = "editor")]
impl<T> imgui_inspect::InspectRenderStruct<CloneComponentPrototype<T>>
    for CloneComponentPrototype<T>
where
    T: Component + Clone + InspectRenderStruct<T>,
{
    fn render(
        data: &[&CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &imgui_inspect::InspectArgsStruct,
    ) {
        let mut r = vec![];
        for d in data {
            let v = d.data();
            r.push(v);
        }

        <T as imgui_inspect::InspectRenderStruct<T>>::render(r.as_slice(), label, ui, args)
    }

    fn render_mut(
        data: &mut [&mut CloneComponentPrototype<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsStruct,
    ) -> bool {
        let mut r = vec![];
        for d in data {
            let v = d.data_mut();
            r.push(v);
        }

        <T as imgui_inspect::InspectRenderStruct<T>>::render_mut(r.as_mut_slice(), label, ui, args)
    }
}

impl<T: Component + Clone + Serialize + DeserializeOwned>
    ComponentPrototypeSerializer<CloneComponentPrototype<T>> for CloneComponentPrototype<T>
{
    fn serialize(
        prototype: &CloneComponentPrototype<T>,
    ) -> Result<serde_json::Value, failure::Error> {
        Ok(serde_json::to_value(prototype.inner.data())?)
    }

    fn deserialize(data: serde_json::Value) -> Result<CloneComponentPrototype<T>, failure::Error> {
        Ok(CloneComponentPrototype::<T>::new(serde_json::from_value(
            data,
        )?))
    }
}

#[cfg(feature = "editor")]
impl<T: Component + Clone + SelectableComponentPrototype<T>>
    SelectableComponentPrototype<CloneComponentPrototype<T>> for CloneComponentPrototype<T>
{
    fn create_selection_shape(
        data: &CloneComponentPrototype<T>,
    ) -> (
        ncollide2d::math::Isometry<f32>,
        ncollide2d::shape::ShapeHandle<f32>,
    ) {
        T::create_selection_shape(&data.inner.data())
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
