use crate::Component;
use crate::ComponentFactory;
use crate::ComponentPrototype;
use crate::EntityHandle;
use crate::EntitySet;
use crate::ResourceMap;
use named_type::NamedType;

use std::collections::VecDeque;

/// Creates a component for an entity by cloning it. This is a basic implementation for unit tests and
/// examples. Downstream code would likely want to duplicate or wrap this so that other cross-cutting
/// functionality (i.e. serialization, editing) can be implemented
#[derive(Clone, NamedType)]
pub struct BasicComponentPrototype<T: Component + Clone> {
    data: T,
}

impl<T: Component + Clone> BasicComponentPrototype<T> {
    pub fn new(data: T) -> Self {
        BasicComponentPrototype::<T> { data }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Component + Clone> ComponentPrototype for BasicComponentPrototype<T> {
    type Factory = BasicComponentFactory<T>;
}

//
// Factory for clone components
//
pub struct BasicComponentFactory<T: Component> {
    prototypes: VecDeque<(EntityHandle, T)>,
}

impl<T: Component> BasicComponentFactory<T> {
    pub fn new() -> Self {
        BasicComponentFactory::<T> {
            prototypes: VecDeque::new(),
        }
    }
}

impl<T: Component + Clone> ComponentFactory<BasicComponentPrototype<T>>
    for BasicComponentFactory<T>
{
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &BasicComponentPrototype<T>,
    ) {
        self.prototypes
            .push_back((entity_handle.clone(), prototype.data.clone()));
    }

    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        let mut storage = resource_map.fetch_mut::<<T as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                entity.add_component(&mut *storage, data);
            }
        }
    }
}
