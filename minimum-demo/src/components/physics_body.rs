use minimum::component::{ComponentCreateQueueFlushListener, SlabComponentStorage};
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::ResourceMap;
use minimum::{Component, ComponentStorage};
use serde::{Deserialize, Serialize};

use nphysics2d::object::BodyHandle;
use nphysics2d::object::ColliderDesc;
use nphysics2d::object::RigidBodyDesc;

use crate::components::PositionComponent;
use crate::framework::inspect::common_types::*;
use crate::framework::FrameworkComponentPrototype;
use named_type::NamedType;
use std::collections::VecDeque;

#[derive(Debug, NamedType)]
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
    type Storage = SlabComponentStorage<Self>;
}

//
// The free handler ensures that when an entity is destroyed, its body components get cleaned up
//
pub struct PhysicsBodyComponentFreeHandler {}

impl minimum::component::ComponentFreeHandler<PhysicsBodyComponent>
    for PhysicsBodyComponentFreeHandler
{
    fn on_entities_free(
        resource_map: &minimum::ResourceMap,
        entity_handles: &[minimum::EntityHandle],
        storage: &mut <PhysicsBodyComponent as Component>::Storage,
    ) {
        let mut physics_manager = resource_map.fetch_mut::<crate::resources::PhysicsManager>();
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
#[derive(Clone, NamedType, Inspect)]
pub struct PhysicsBodyComponentPrototypeCustom {
    #[inspect(skip)]
    desc: std::sync::Arc<PhysicsBodyComponentDesc>,
}

impl PhysicsBodyComponentPrototypeCustom {
    pub fn new(desc: PhysicsBodyComponentDesc) -> Self {
        PhysicsBodyComponentPrototypeCustom {
            desc: std::sync::Arc::new(desc),
        }
    }
}

impl ComponentPrototype for PhysicsBodyComponentPrototypeCustom {
    type Factory = PhysicsBodyComponentFactory;
}

impl FrameworkComponentPrototype for PhysicsBodyComponentPrototypeCustom {}

//
// Creates a component for an entity by copying it
//
#[derive(Clone, NamedType, Inspect, Serialize, Deserialize)]
pub struct PhysicsBodyComponentPrototypeBox {
    #[inspect(proxy_type = "ImGlmVec2")]
    size: glm::Vec2,

    //TODO: Support more than one!
    collision_group_membership: usize,
}

impl PhysicsBodyComponentPrototypeBox {
    pub fn new(size: glm::Vec2, collision_group_membership: usize) -> Self {
        PhysicsBodyComponentPrototypeBox {
            size,
            collision_group_membership,
        }
    }
}

impl ComponentPrototype for PhysicsBodyComponentPrototypeBox {
    type Factory = PhysicsBodyComponentFactory;
}

impl FrameworkComponentPrototype for PhysicsBodyComponentPrototypeBox {}

enum QueuedPhysicsBodyPrototypes {
    Box(PhysicsBodyComponentPrototypeBox),
    Custom(PhysicsBodyComponentPrototypeCustom),
}

//
// Factory for PhysicsBody components
//
pub struct PhysicsBodyComponentFactory {
    prototypes: VecDeque<(EntityHandle, QueuedPhysicsBodyPrototypes)>,
}

impl PhysicsBodyComponentFactory {
    pub fn new() -> Self {
        PhysicsBodyComponentFactory {
            prototypes: VecDeque::new(),
        }
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototypeCustom> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototypeCustom,
    ) {
        self.prototypes.push_back((
            entity_handle.clone(),
            QueuedPhysicsBodyPrototypes::Custom(prototype.clone()),
        ));
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototypeBox> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototypeBox,
    ) {
        self.prototypes.push_back((
            entity_handle.clone(),
            QueuedPhysicsBodyPrototypes::Box(prototype.clone()),
        ));
    }
}

impl ComponentCreateQueueFlushListener for PhysicsBodyComponentFactory {
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        //TODO: Either need two-phase entity construction or deterministic construct order.
        let position = resource_map.fetch::<<PositionComponent as Component>::Storage>();

        let mut physics = resource_map.fetch_mut::<crate::resources::PhysicsManager>();
        let mut storage = resource_map.fetch_mut::<<PhysicsBodyComponent as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                let center: glm::Vec2 =
                    if let Some(p) = entity.get_component::<PositionComponent>(&*position) {
                        p.position()
                    } else {
                        glm::zero()
                    };

                match data {
                    QueuedPhysicsBodyPrototypes::Box(data) => {
                        use ncollide2d::shape::{Cuboid, ShapeHandle};
                        use nphysics2d::material::{BasicMaterial, MaterialHandle};
                        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

                        println!("create box {:?}", data.size);

                        let shape = ShapeHandle::new(Cuboid::new(data.size / 2.0));

                        let collider_desc = ColliderDesc::new(shape)
                            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
                            .collision_groups(
                                ncollide2d::world::CollisionGroups::new()
                                    .with_membership(&[data.collision_group_membership]),
                            );

                        let body_desc = RigidBodyDesc::new()
                            .translation(center)
                            .kinematic_rotation(false)
                            .collider(&collider_desc);

                        let body = physics.world_mut().add_body(&body_desc);
                        entity
                            .add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()));
                    }

                    QueuedPhysicsBodyPrototypes::Custom(data) => {
                        let body = physics.world_mut().add_body(data.desc.rigid_body_desc());
                        entity
                            .add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()));
                    }
                }
            }
        }
    }
}
