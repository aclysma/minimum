use super::Component;
use super::ComponentFactory;
use super::ComponentPrototype;
use super::ComponentStorage;
use named_type::NamedType;
use std::marker::PhantomData;

use crate::{EntityHandle, EntitySet, Resource, ResourceMap};

/// Handler can be implemented to run custom logic just before entities are destroyed
pub trait ComponentFreeHandler<T: Component>: Send + Sync {
    fn on_entities_free(
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        storage: &mut <T as Component>::Storage,
    );
}

/// Default handler, does nothing but removes the component from the storage
struct DefaultFreeHandler {}

impl<T: Component> ComponentFreeHandler<T> for DefaultFreeHandler {
    fn on_entities_free(
        _resource_map: &ResourceMap,
        _entity_handles: &[EntityHandle],
        _storage: &mut <T as Component>::Storage,
    ) {
        // The default behavior does nothing extra
    }
}

/// Custom handler, calls on_entities_free on the given type
struct CustomFreeHandler<T: Component, F: ComponentFreeHandler<T>> {
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<F>,
}

impl<T, F> ComponentFreeHandler<T> for CustomFreeHandler<T, F>
where
    T: Component,
    F: ComponentFreeHandler<T> + 'static,
{
    fn on_entities_free(
        resource_map: &ResourceMap,
        entity_handles: &[EntityHandle],
        storage: &mut <T as Component>::Storage,
    ) {
        // The custom behavior proxies the call to the provided type
        F::on_entities_free(resource_map, entity_handles, storage);
    }
}

/// Interface for a registered component type
trait RegisteredComponentTrait: Send + Sync {
    fn on_entities_free(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle]);
}

/// Represents a component that has been registered.
struct RegisteredComponent<T, F>
where
    T: Component,
    F: ComponentFreeHandler<T>,
{
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<F>,
}

impl<T, F> RegisteredComponent<T, F>
where
    T: Component,
    F: ComponentFreeHandler<T>,
{
    fn new() -> Self {
        RegisteredComponent {
            phantom_data1: PhantomData,
            phantom_data2: PhantomData,
        }
    }
}

impl<T, F> RegisteredComponentTrait for RegisteredComponent<T, F>
where
    T: Component,
    F: ComponentFreeHandler<T>,
{
    fn on_entities_free(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle]) {
        let mut storage = resource_map.fetch_mut::<T::Storage>();
        F::on_entities_free(resource_map, entity_handles, &mut *storage);

        for entity_handle in entity_handles {
            storage.free_if_exists(entity_handle);
        }
    }
}

/// Used to walk across all component factories and flush pending creates
trait RegisteredComponentFactoryTrait: Resource {
    fn flush_creates(&self, resource_map: &ResourceMap, entity_set: &EntitySet);
}

/// Represents a registered component factory, used to allow iterating all registered factories and
/// calling `flush_creates` on them
#[derive(NamedType)]
pub struct RegisteredComponentFactory<P, F>
where
    P: ComponentPrototype,
    F: ComponentFactory<P>,
{
    phantom_data1: PhantomData<P>,
    phantom_data2: PhantomData<F>,
}

impl<P, F> RegisteredComponentFactory<P, F>
where
    P: ComponentPrototype,
    F: ComponentFactory<P>,
{
    fn new() -> Self {
        RegisteredComponentFactory {
            phantom_data1: PhantomData,
            phantom_data2: PhantomData,
        }
    }
}

impl<P, F> RegisteredComponentFactoryTrait for RegisteredComponentFactory<P, F>
where
    P: ComponentPrototype,
    F: ComponentFactory<P>,
{
    fn flush_creates(&self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        let mut factory = resource_map.fetch_mut::<F>();
        factory.flush_creates(resource_map, entity_set);
    }
}

/// Allows registration of component types. Registering a type allows us to iterate and execute code
/// for every type (mostly to handle deferred create/destroy)
pub struct ComponentRegistry {
    registered_components: Vec<Box<dyn RegisteredComponentTrait>>,
    registered_factories: Vec<Box<dyn RegisteredComponentFactoryTrait>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        ComponentRegistry {
            registered_components: vec![],
            registered_factories: vec![],
        }
    }

    /// All components must be registered
    pub fn register_component<T: Component + 'static>(&mut self) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T, DefaultFreeHandler>::new()));
    }

    //TODO: This API is sub-par, a builder might be better?
    /// If the component has its own custom free handler, call this rather than `register_component`
    pub fn register_component_with_free_handler<
        T: Component + 'static,
        F: ComponentFreeHandler<T> + 'static,
    >(
        &mut self,
    ) {
        self.registered_components.push(Box::new(
            RegisteredComponent::<T, CustomFreeHandler<T, F>>::new(),
        ));
    }

    /// All factories must be registered. (Multiple components may exist for each component type. This
    /// allows multiple distinct prototype types to be supported per component type)
    pub fn register_component_factory<P: ComponentPrototype, F: ComponentFactory<P>>(&mut self) {
        self.registered_factories
            .push(Box::new(RegisteredComponentFactory::<P, F>::new()));
    }

    /// Handle any deferred component allocations
    pub fn on_flush_creates(&self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        for rf in &self.registered_factories {
            rf.flush_creates(resource_map, entity_set);
        }
    }

    /// Free components of every type for the given entities
    pub fn on_entities_free(&self, resource_map: &ResourceMap, entity_handles: &[EntityHandle]) {
        for rc in &self.registered_components {
            rc.on_entities_free(resource_map, entity_handles);
        }
    }
}
