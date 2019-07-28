use minimum::component::SlabComponentStorage;
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::World;
use minimum::{Component, ComponentStorage};

use nphysics2d::object::BodyHandle;
use nphysics2d::object::ColliderDesc;
use nphysics2d::object::RigidBodyDesc;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct PhysicsBodyComponent {
    body_handle: BodyHandle,
}

//
// Holds a handle to a physics body
// - It's always possible to directly create this component using a body_handle, but you need to have
//   a write lock on the component storage
// - If you'd rather defer construction, you could instead create a PhysicsBodyComponentPrototype, which
//   takes an nphysics RigidBodyDesc. The PhysicsBodyComponentFactory will create the body for you
//   and attach it to the entity
// - If the entity owning the component is cleaned up, PhysicsBodyComponentFreeHandler will handle
//   destroying the physics body
//
impl PhysicsBodyComponent {
    pub fn new(body_handle: BodyHandle) -> Self {
        PhysicsBodyComponent { body_handle }
    }

    pub fn body_handle(&self) -> BodyHandle {
        self.body_handle
    }
}

impl minimum::Component for PhysicsBodyComponent {
    type Storage = SlabComponentStorage<PhysicsBodyComponent>;
}

//
// The free handler ensures that when an entity is destroyed, its body components get cleaned up
//
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

//
// This is a wrapper for RigidBodyDesc. RigidBodyDesc requires a lifetime and holds refs to
// ColliderDescs. I think it will be a bit easier to allow the body to psuedo-own its own colliders.
// This object owns the RigidBodyDesc and each ColliderDesc, and allows the RigidBodyDesc to have
// references to the colliders.
//
// Another approach would probably be to make separate, more serialization-friendly description
// structures. This has the added advantage that these objects could be expressed in data files.
//
pub struct PhysicsBodyComponentDesc {
    collider_desc: Vec<Box<ColliderDesc<f32>>>,
    rigid_body_desc: RigidBodyDesc<'static, f32>,
}

impl PhysicsBodyComponentDesc {
    pub fn new(rigid_body_desc: RigidBodyDesc<'static, f32>) -> Self {
        PhysicsBodyComponentDesc {
            collider_desc: Vec::new(),
            rigid_body_desc,
        }
    }

    pub fn add_collider(&mut self, collider_desc: ColliderDesc<f32>) {
        // Box the collider
        let collider_boxed = Box::new(collider_desc);

        // Turn the box into a box and a ref to the same box. The new box will solely be responsible
        // for cleaning up the memory. The reference will be given to the body. Because the
        // collider and body desc have the same lifetimes, the reference will always be valid for
        // the lifetime of the body description
        let collider_ptr: *mut ColliderDesc<f32> = Box::into_raw(collider_boxed);
        let collider_ref = unsafe { &*collider_ptr };
        let collider_boxed = unsafe { Box::from_raw(collider_ptr) };

        // Push the box to ensure the memory gets cleaned up when this struct is cleaned up
        self.collider_desc.push(collider_boxed);
        self.rigid_body_desc.add_collider(collider_ref);
    }

    pub fn rigid_body_desc(&self) -> &RigidBodyDesc<f32> {
        &self.rigid_body_desc
    }
}

//
// Creates a component for an entity by copying it
//
#[derive(Clone)]
pub struct PhysicsBodyComponentPrototype {
    desc: std::sync::Arc<PhysicsBodyComponentDesc>,
}

impl<'a> PhysicsBodyComponentPrototype {
    pub fn new(desc: PhysicsBodyComponentDesc) -> Self {
        PhysicsBodyComponentPrototype {
            desc: std::sync::Arc::new(desc),
        }
    }
}

impl<'a> ComponentPrototype for PhysicsBodyComponentPrototype {
    type Factory = PhysicsBodyComponentFactory;
}

//
// Factory for PhysicsBody components
//
pub struct PhysicsBodyComponentFactory {
    prototypes: VecDeque<(EntityHandle, PhysicsBodyComponentPrototype)>,
}

impl PhysicsBodyComponentFactory {
    pub fn new() -> Self {
        PhysicsBodyComponentFactory {
            prototypes: VecDeque::new(),
        }
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototype> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototype,
    ) {
        self.prototypes
            .push_back((entity_handle.clone(), prototype.clone()));
    }

    fn flush_creates(&mut self, world: &World, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        let mut physics = world.fetch_mut::<crate::resources::PhysicsManager>();
        let mut storage = world.fetch_mut::<<PhysicsBodyComponent as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                let body = physics.world_mut().add_body(data.desc.rigid_body_desc());
                entity.add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()));
            }
        }
    }
}
