use super::Component;
use super::ComponentStorage;
use super::EntityHandle;
use std::marker::PhantomData;

use crate::systems;
use systems::World;

//
// Handler can be implemented to run custom logic when entities are being destroyed
//
pub trait ComponentFreeHandler<T : Component> : Send + Sync {
    fn on_entities_free(world: &World, entity_handles: &[EntityHandle], storage: &mut <T as Component>::Storage);
}

//
// Default handler, does nothing but removes the component from the storage
//
struct DefaultFreeHandler {

}

impl<T : Component> ComponentFreeHandler<T> for DefaultFreeHandler {
    fn on_entities_free(world: &World, entity_handles: &[EntityHandle], storage: &mut <T as Component>::Storage) {
        // The default behavior does nothing extra
    }
}

//
// Custom handler, calls on_entities_free on the given type
//
struct CustomFreeHandler<T : Component, F : ComponentFreeHandler<T>> {
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<F>
}

impl<T, F> CustomFreeHandler<T, F>
where
    T : Component,
    F : ComponentFreeHandler<T>
{
    fn new() -> Self {
        CustomFreeHandler::<T, F> {
            phantom_data1: PhantomData,
            phantom_data2: PhantomData
        }
    }
}

impl<T, F> ComponentFreeHandler<T> for CustomFreeHandler<T, F>
where
    T : Component,
    F : ComponentFreeHandler<T> + 'static
{
    fn on_entities_free(world: &World, entity_handles: &[EntityHandle], storage: &mut <T as Component>::Storage) {
        // The custom behavior proxies the call to the provided type
        F::on_entities_free(world, entity_handles, storage);
    }
}

//
// Interface for a registered component type
//
trait RegisteredComponentTrait : Send + Sync {
    fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]);
}

pub struct RegisteredComponent<T, F>
    where
        T: Component,
        F: ComponentFreeHandler<T>
{
    phantom_data1: PhantomData<T>,
    phantom_data2: PhantomData<F>

}

impl<T, F> RegisteredComponent<T, F>
    where
        T: Component,
        F: ComponentFreeHandler<T>
{
    fn new() -> Self {
        RegisteredComponent {
            phantom_data1: PhantomData,
            phantom_data2: PhantomData
        }
    }
}

impl<T, F> RegisteredComponentTrait for RegisteredComponent<T, F>
    where
        T: Component,
        F: ComponentFreeHandler<T>
{
    fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        let mut storage = world.fetch_mut::<T::Storage>();
        F::on_entities_free(world, entity_handles, &mut *storage);

        for entity_handle in entity_handles {
            storage.free_if_exists(entity_handle);
        }
    }
}

pub struct ComponentRegistry {
    registered_components: Vec<Box<RegisteredComponentTrait>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        ComponentRegistry {
            registered_components: vec![],
        }
    }

    pub fn register_component<T: Component + 'static>(&mut self) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T, DefaultFreeHandler>::new()));
    }

    pub fn register_component_type_with_free_handler<T: Component + 'static, F: ComponentFreeHandler<T> + 'static>(&mut self) {
        self.registered_components
            .push(Box::new(RegisteredComponent::<T, CustomFreeHandler<T, F>>::new()));
    }

    pub fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        for rc in &self.registered_components {
            rc.on_entities_free(world, entity_handles);
        }
    }
}
