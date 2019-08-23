//! The types here are for examples, unit tests, and demonstration. But, there's no reason you *must*
//! use them and in fact, implementing your own version of this will give you more flexibility.

use crate::Component;
use crate::ComponentFactory;
use crate::ComponentPrototype;
use crate::EntityHandle;
use crate::EntitySet;
use crate::ResourceMap;
use named_type::NamedType;

use std::collections::VecDeque;

/// Represents a component that will be created in the future by cloning `data`.
///
/// This is a basic implementation that may be used, but downstream code would likely want to
/// duplicate or wrap this so that other cross-cutting functionality (i.e. serialization, editing)
/// can be implemented
#[derive(Clone, NamedType)]
pub struct BasicComponentPrototype<T: Component + Clone> {
    /// This data will be cloned and placed on the entity
    data: T,
}

impl<T: Component + Clone> BasicComponentPrototype<T> {
    /// Creates the prototype
    pub fn new(data: T) -> Self {
        BasicComponentPrototype::<T> { data }
    }

    /// Get the data that will be cloned to create the component
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get the data that will be cloned to create the component
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Component + Clone> ComponentPrototype for BasicComponentPrototype<T> {
    type Factory = BasicComponentFactory<T>;
}

/// Factory for clone components
pub struct BasicComponentFactory<T: Component> {
    /// Queue of BasicComponentPrototypes to create
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
    /// Store the component prototype to be created later in the frame
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &BasicComponentPrototype<T>,
    ) {
        self.prototypes
            .push_back((entity_handle.clone(), prototype.data.clone()));
    }

    /// Kicks off creating all deferred components
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        // Bail if nothing to create
        if self.prototypes.is_empty() {
            return;
        }

        // Get write access to T's storage
        let mut storage = resource_map.fetch_mut::<<T as Component>::Storage>();

        // Drain the queue, converting all queued prototypes to real components
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                entity.add_component(&mut *storage, data);
            }
        }
    }
}
