use minimum::component::SlabComponentStorage;
use minimum::{Component, ComponentStorage};
use nphysics2d::object::BodyHandle;

#[derive(Debug)]
pub struct PhysicsBodyComponent {
    body_handle: BodyHandle,
}

impl PhysicsBodyComponent {
    pub fn new(body_handle: BodyHandle) -> Self {
        PhysicsBodyComponent { body_handle }
    }

    //TODO: Need to have a handler to destroy the body
    pub fn body_handle(&self) -> BodyHandle {
        self.body_handle
    }
}

impl minimum::Component for PhysicsBodyComponent {
    type Storage = SlabComponentStorage<PhysicsBodyComponent>;
}

pub struct PhysicsBodyComponentFreeHandler {}

impl minimum::component::ComponentFreeHandler<PhysicsBodyComponent>
    for PhysicsBodyComponentFreeHandler
{
    fn on_entities_free(
        world: &minimum::World,
        entity_handles: &[minimum::EntityHandle],
        storage: &mut <PhysicsBodyComponent as Component>::Storage,
    ) {
        let mut physics_manager = world.fetch_mut::<crate::resources::PhysicsManager>();
        let physics_world: &mut nphysics2d::world::World<f32> = physics_manager.world_mut();

        for entity_handle in entity_handles {
            if let Some(c) = storage.get_mut(&entity_handle) {
                physics_world.remove_bodies(&[c.body_handle]);
            }
        }
    }
}
