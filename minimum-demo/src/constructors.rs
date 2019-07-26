use crate::components;
use crate::resources;
use minimum::component::CloneComponentPrototype;
use minimum::entity::EntityPrototype;
use minimum::systems::DataRequirement;
use minimum::{Read, World, Write};
use std::collections::VecDeque;

pub fn create_player(entity_factory: &mut minimum::EntityFactory) {
    let entity_prototype = EntityPrototype::new(vec![
        Box::new(CloneComponentPrototype::new(
            components::PlayerComponent::new(),
        )),
        Box::new(CloneComponentPrototype::new(
            components::PositionComponent::new(glm::zero()),
        )),
        Box::new(CloneComponentPrototype::new(
            components::DebugDrawCircleComponent::new(15.0, glm::Vec4::new(0.0, 1.0, 0.0, 1.0)),
        )),
    ]);
    entity_factory.enqueue_create(entity_prototype);
}

type BulletFactoryResources = (
    Read<resources::TimeState>,
    Write<resources::PhysicsManager>,
    Write<minimum::EntityFactory>,
);

pub fn create_bullet(
    prototype: BulletPrototype,
    resources: <BulletFactoryResources as minimum::systems::DataRequirement>::Borrow,
) {
    let radius = 5.0;
    let color = glm::Vec4::new(1.0, 0.0, 0.0, 1.0);

    let (time_state, mut physics_manager, mut entity_factory) = resources;

    let body_handle = {
        use ncollide2d::shape::{Ball, ShapeHandle};
        use nphysics2d::material::{BasicMaterial, MaterialHandle};
        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

        let shape = ShapeHandle::new(Ball::new(radius));
        let collider_desc = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
            .name("bullet".to_string());

        let body = RigidBodyDesc::new()
            .translation(prototype.position)
            .mass(1000.0)
            .collider(&collider_desc)
            .kinematic_rotation(false)
            .name("bullet".to_string())
            .build(physics_manager.world_mut());

        body.handle()
    };

    {
        let entity_prototype = EntityPrototype::new(vec![
            Box::new(CloneComponentPrototype::new(
                components::PositionComponent::new(prototype.position),
            )),
            Box::new(CloneComponentPrototype::new(
                components::VelocityComponent::new(prototype.velocity),
            )),
            Box::new(CloneComponentPrototype::new(
                components::PhysicsBodyComponent::new(body_handle),
            )),
            Box::new(CloneComponentPrototype::new(
                components::DebugDrawCircleComponent::new(radius, color),
            )),
            Box::new(CloneComponentPrototype::new(
                components::BulletComponent::new(),
            )),
            Box::new(CloneComponentPrototype::new(
                components::FreeAtTimeComponent::new(
                    time_state.frame_start_instant + std::time::Duration::from_secs(4),
                ),
            )),
        ]);
        entity_factory.enqueue_create(entity_prototype);
    }
}

pub struct BulletPrototype {
    position: glm::Vec2,
    velocity: glm::Vec2,
}

impl BulletPrototype {
    pub fn new(position: glm::Vec2, velocity: glm::Vec2) -> Self {
        BulletPrototype { position, velocity }
    }
}

pub struct BulletFactory {
    prototypes: VecDeque<BulletPrototype>,
}

impl BulletFactory {
    pub fn new() -> Self {
        BulletFactory {
            prototypes: VecDeque::new(),
        }
    }

    pub fn enqueue_create(&mut self, bullet_prototype: BulletPrototype) {
        self.prototypes.push_back(bullet_prototype);
    }

    pub fn flush_creates(&mut self, world: &World) {
        if self.prototypes.is_empty() {
            return;
        }

        for p in self.prototypes.drain(..) {
            //TODO: fetch per object is bad
            let resources = <BulletFactoryResources as DataRequirement>::fetch(world);
            create_bullet(p, resources);
        }
    }
}
