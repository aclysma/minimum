use crate::Component;
use crate::EntityHandle;
use crate::EntitySet;
use crate::Resource;
use crate::World;
use std::collections::VecDeque;

//TODO: Change naming from prototype to definition

//
// General component prototype.. it knows the factory type that can construct it, and queue_create()
// is implemented so that dynamic dispatch to enqueue the prototype is supported
//
pub trait ComponentPrototype: Sized + Send + Sync + 'static {
    type Factory: ComponentFactory<Self>;

    fn enqueue_create(&self, world: &World, entity_handle: &EntityHandle) {
        let mut factory = world.fetch_mut::<Self::Factory>();
        factory.enqueue_create(entity_handle, self);
    }
}

//
// General purpose factory. Given a prototype, it can put a component on an entity
//
pub trait ComponentFactory<P: ComponentPrototype>: Resource {
    fn enqueue_create(&mut self, entity_handle: &EntityHandle, prototype: &P);

    fn flush_creates(&mut self, world: &World, entity_set: &EntitySet);
}

//
// Creates a component for an entity by copying it
//
pub struct CloneComponentPrototype<T: Component + Clone> {
    clone: T,
}

impl<T: Component + Clone> CloneComponentPrototype<T> {
    pub fn new(clone: T) -> Self {
        CloneComponentPrototype::<T> { clone }
    }
}

impl<T: Component + Clone> ComponentPrototype for CloneComponentPrototype<T> {
    type Factory = CloneComponentFactory<T>;
}

//
// Factory for clone components
//
pub struct CloneComponentFactory<T: Component> {
    prototypes: VecDeque<(EntityHandle, T)>,
}

impl<T: Component> CloneComponentFactory<T> {
    pub fn new() -> Self {
        CloneComponentFactory::<T> {
            prototypes: VecDeque::new(),
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
        self.prototypes
            .push_back((entity_handle.clone(), prototype.clone.clone()));
    }

    fn flush_creates(&mut self, world: &World, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        let mut storage = world.fetch_mut::<<T as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                entity.add_component(&mut *storage, data);
            }
        }
    }
}
