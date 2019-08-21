use crate::Component;
use crate::EntityHandle;
use crate::EntitySet;
use crate::Resource;
use crate::ResourceMap;
use named_type::NamedType;
use std::collections::VecDeque;

//
// EntityPrototype wants to hold a list of component prototypes, but this trait has an
// associated type and it's not currently possible to have a Box<dyn Trait> without specifying the
// associated type.
//
// Ideally this type would not exist and we would use ComponentPrototype everywhere
//
pub trait ComponentCreator: Sync + Send {
    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle);
}

//
// General component prototype.. it knows the factory type that can construct it, and queue_create()
// is implemented so that dynamic dispatch to enqueue the prototype is supported
//
pub trait ComponentPrototype: ComponentCreator + Sized + Send + Sync + 'static {
    type Factory: ComponentFactory<Self>;

    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle) {
        let mut factory = resource_map.fetch_mut::<Self::Factory>();
        factory.enqueue_create(entity_handle, self);
    }
}

impl<T> ComponentCreator for T
    where
        T: ComponentPrototype + Sync + Send,
{
    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle) {
        <T as ComponentPrototype>::enqueue_create(&self, resource_map, entity_handle);
    }
}

//
// General purpose factory. Given a prototype, it can put a component on an entity
//
pub trait ComponentFactory<P: ComponentPrototype>: Resource {
    fn enqueue_create(&mut self, entity_handle: &EntityHandle, prototype: &P);

    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet);
}
