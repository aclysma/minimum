use super::Component;
use super::ComponentFactory;
use super::ComponentPrototype;
use super::ComponentStorage;
use std::marker::PhantomData;

use crate::{EntityHandle, EntitySet, Resource, World};
use crate::entity::EntityRef;

//
// Handler can be implemented to run custom logic when entities are being destroyed
//
pub trait ComponentFreeHandler<T: Component>: Send + Sync {
    fn on_entities_free(
        world: &World,
        entity_handles: &[EntityHandle],
        storage: &mut <T as Component>::Storage,
    );
}

//
// Default handler, does nothing but removes the component from the storage
//
struct DefaultFreeHandler {}

impl<T: Component> ComponentFreeHandler<T> for DefaultFreeHandler {
    fn on_entities_free(
        _world: &World,
        _entity_handles: &[EntityHandle],
        _storage: &mut <T as Component>::Storage,
    ) {
        // The default behavior does nothing extra
    }
}

//
// Custom handler, calls on_entities_free on the given type
//
struct CustomFreeHandler<T: Component, F: ComponentFreeHandler<T>> {
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<F>,
}

//impl<T, F> CustomFreeHandler<T, F>
//where
//    T: Component,
//    F: ComponentFreeHandler<T>,
//{
//    //    fn new() -> Self {
//    //        CustomFreeHandler::<T, F> {
//    //            phantom_data1: PhantomData,
//    //            phantom_data2: PhantomData
//    //        }
//    //    }
//}

impl<T, F> ComponentFreeHandler<T> for CustomFreeHandler<T, F>
where
    T: Component,
    F: ComponentFreeHandler<T> + 'static,
{
    fn on_entities_free(
        world: &World,
        entity_handles: &[EntityHandle],
        storage: &mut <T as Component>::Storage,
    ) {
        // The custom behavior proxies the call to the provided type
        F::on_entities_free(world, entity_handles, storage);
    }
}

//
// Interface for a registered component type
//
trait RegisteredComponentTrait: Send + Sync {
    fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]);

    fn visit_component(&self, world: &World, entity_handles: &[EntityHandle]);
}

pub struct RegisteredComponent<T, F>
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
    fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        let mut storage = world.fetch_mut::<T::Storage>();
        F::on_entities_free(world, entity_handles, &mut *storage);

        for entity_handle in entity_handles {
            storage.free_if_exists(entity_handle);
        }
    }

    fn visit_component(&self, world: &World, entity_handles: &[EntityHandle]) {
        let storage = world.fetch::<<T as Component>::Storage>();

        for entity_handle in entity_handles {
            let comp = storage.get(&entity_handle);

            if let Some(comp) = comp {
                println!("found a name {}", comp.get_name());
            }
        }
    }
}

//
// Registered component factory
//

trait RegisteredComponentFactoryTrait: Resource {
    fn flush_creates(&self, world: &World, entity_set: &EntitySet);
}

#[derive(typename::TypeName)]
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
    fn flush_creates(&self, world: &World, entity_set: &EntitySet) {
        let mut factory = world.fetch_mut::<F>();
        factory.flush_creates(world, entity_set);
    }
}

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

    pub fn register_component<T: Component + 'static>(&mut self) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T, DefaultFreeHandler>::new()));
    }

    pub fn register_component_type_with_free_handler<
        T: Component + 'static,
        F: ComponentFreeHandler<T> + 'static,
    >(
        &mut self,
    ) {
        self.registered_components.push(Box::new(
            RegisteredComponent::<T, CustomFreeHandler<T, F>>::new(),
        ));
    }

    pub fn register_component_factory<P: ComponentPrototype, F: ComponentFactory<P>>(&mut self) {
        self.registered_factories
            .push(Box::new(RegisteredComponentFactory::<P, F>::new()));
    }

    pub fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        for rc in &self.registered_components {
            rc.on_entities_free(world, entity_handles);
        }
    }

    pub fn visit_components(&self, world: &World, entity_handles: &[EntityHandle]) {
        for rc in &self.registered_components {
            rc.visit_component(world, entity_handles);
        }
    }

    pub fn on_flush_creates(&self, world: &World, entity_set: &EntitySet) {
        for rf in &self.registered_factories {
            rf.flush_creates(world, entity_set);
        }
    }
}
