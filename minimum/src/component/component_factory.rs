use crate::EntityHandle;
use crate::EntitySet;
use crate::Resource;
use crate::ResourceMap;

//TODO: Move this back to BasicEntityPrototype
/// A minimal interface for attaching a component specified by a prototype to an entity
///
/// BasicEntityPrototype wants to hold a list of ComponentPrototype, but that trait has an
/// associated type and it's not currently possible to have a Box<dyn Trait> without specifying the
/// associated type.
///
/// Ideally this type would not exist and we would use ComponentPrototype directly
pub trait ComponentCreator: Sync + Send {
    fn enqueue_create(&self, resource_map: &ResourceMap, entity_handle: &EntityHandle);
}

/// General component prototype.. it knows the factory type that can construct it, and queue_create()
/// is implemented so that we can simply dynamic dispatch to enqueue the prototype
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

/// General purpose factory. Given a prototype, it can put a component on an entity
///
/// This is separated from ComponentCreateQueueFlushListener so that a single factory
/// can support multiple prototypes
pub trait ComponentFactory<P: ComponentPrototype> : Resource {
    /// Enqueue a prototype to put on the given entity later when `flush_creates` is called
    fn enqueue_create(&mut self, entity_handle: &EntityHandle, prototype: &P);
}

/// General purpose factory. Given a prototype, it can put a component on an entity
pub trait ComponentCreateQueueFlushListener : Resource {
    /// Puts all queue components on their respective entities. If the entity does not exist, the
    /// component is not created
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet);
}
