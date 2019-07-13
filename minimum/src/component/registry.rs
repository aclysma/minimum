
use super::EntityHandle;
use super::Component;
use super::ComponentStorage;
use super::ComponentStorageBase;

use crate::async_dispatcher;
use crate::systems;
use systems::World;

//TODO: Can this be done with a trait?
type OnEntitiesFreeCb = fn(&World, &[EntityHandle]);
type CheckComponentExists = fn(&World, &EntityHandle);

pub struct RegisteredComponent {
    on_entities_free_cb: OnEntitiesFreeCb
}

impl RegisteredComponent {
    fn new(on_entities_free_cb: OnEntitiesFreeCb) -> Self {
        RegisteredComponent {
            on_entities_free_cb
        }
    }

    fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        (self.on_entities_free_cb)(world, entity_handles);
    }
}

pub struct ComponentRegistry {
    registered_components: Vec<RegisteredComponent>
}

impl ComponentRegistry {
    pub fn new() -> Self{
        ComponentRegistry {
            registered_components: vec![]
        }
    }

    pub fn register_component<T : Component + 'static>(&mut self) {
        let callback = |world: &World, entity_handles: &[EntityHandle]| {
            let mut storage = world.fetch_mut::<T::Storage>();
            for entity_handle in entity_handles {
                storage.free_if_exists(entity_handle);
            }
        };

        self.registered_components.push(RegisteredComponent::new(callback));
    }

    pub fn on_entities_free(&self, world: &World, entity_handles: &[EntityHandle]) {
        println!("on_entities_free {:?}", entity_handles);
        for rc in &self.registered_components {
            rc.on_entities_free(world, entity_handles);
        }
    }
}
