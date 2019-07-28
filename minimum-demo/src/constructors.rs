use crate::components;
use crate::resources;
use minimum::component::CloneComponentPrototype;
use minimum::entity::EntityPrototype;
use minimum::{Read, Write};

use components::PhysicsBodyComponentPrototype;
use rand::Rng;

const COLLISION_GROUP_PLAYER: usize = 0;
const COLLISION_GROUP_BULLETS: usize = 1;
const COLLISION_GROUP_WALL: usize = 2;

pub fn create_wall(
    center: glm::Vec2,
    size: glm::Vec2,
    entity_factory: &mut minimum::EntityFactory,
) {
    let color = glm::Vec4::new(0.0, 1.0, 1.0, 1.0);

    let body_component_desc = {
        use ncollide2d::shape::{Cuboid, ShapeHandle};
        use nphysics2d::material::{BasicMaterial, MaterialHandle};
        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

        let shape = ShapeHandle::new(Cuboid::new(size / 2.0));
        let collider_desc = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
            .collision_groups(
                ncollide2d::world::CollisionGroups::new().with_membership(&[COLLISION_GROUP_WALL]),
            )
            .name("wall".to_string());

        let body_desc = RigidBodyDesc::new()
            .translation(center)
            .kinematic_rotation(false)
            .name("wall".to_string());

        let mut body_component_desc = components::PhysicsBodyComponentDesc::new(body_desc);
        body_component_desc.add_collider(collider_desc);

        body_component_desc
    };

    let entity_prototype = EntityPrototype::new(vec![
        Box::new(CloneComponentPrototype::new(
            components::PositionComponent::new(center),
        )),
        Box::new(CloneComponentPrototype::new(
            components::DebugDrawRectComponent::new(size, color),
        )),
        Box::new(PhysicsBodyComponentPrototype::new(body_component_desc)),
    ]);
    entity_factory.enqueue_create(entity_prototype);
}

pub fn create_player(entity_factory: &mut minimum::EntityFactory) {
    let position = glm::zero();
    let radius = 15.0;
    let color = glm::Vec4::new(0.0, 1.0, 0.0, 1.0);

    let body_component_desc = {
        use ncollide2d::shape::{Ball, ShapeHandle};
        use nphysics2d::material::{BasicMaterial, MaterialHandle};
        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

        let shape = ShapeHandle::new(Ball::new(radius));
        let collider_desc = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
            .collision_groups(
                ncollide2d::world::CollisionGroups::new()
                    .with_membership(&[COLLISION_GROUP_PLAYER])
                    .with_whitelist(&[COLLISION_GROUP_WALL]),
            )
            .name("player".to_string());

        let body_desc = RigidBodyDesc::new()
            .translation(position)
            .mass(1000.0)
            .kinematic_rotation(false)
            .name("player".to_string());

        let mut body_component_desc = components::PhysicsBodyComponentDesc::new(body_desc);
        body_component_desc.add_collider(collider_desc);

        body_component_desc
    };

    let entity_prototype = EntityPrototype::new(vec![
        Box::new(CloneComponentPrototype::new(
            components::PlayerComponent::new(),
        )),
        Box::new(CloneComponentPrototype::new(
            components::PositionComponent::new(position),
        )),
        Box::new(CloneComponentPrototype::new(
            components::DebugDrawCircleComponent::new(radius, color),
        )),
        Box::new(PhysicsBodyComponentPrototype::new(body_component_desc)),
    ]);
    entity_factory.enqueue_create(entity_prototype);
}

pub fn create_bullet(
    position: glm::Vec2,
    velocity: glm::Vec2,
    time_state: &resources::TimeState,
    entity_factory: &mut minimum::EntityFactory,
) {
    let radius = 5.0;
    let color = glm::Vec4::new(1.0, 1.0, 0.0, 1.0);
    let lifetime = std::time::Duration::from_secs(4);
    let mut rng = rand::thread_rng();

    let restitution = rng.gen_range(0.8, 1.0);

    let body_component_desc = {
        use ncollide2d::shape::{Ball, ShapeHandle};
        use nphysics2d::material::{BasicMaterial, MaterialHandle};
        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

        let shape = ShapeHandle::new(Ball::new(radius));
        let collider_desc = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(restitution, 0.0)))
            .collision_groups(
                ncollide2d::world::CollisionGroups::new()
                    .with_membership(&[COLLISION_GROUP_BULLETS])
                    .with_blacklist(&[COLLISION_GROUP_BULLETS]),
            )
            .name("bullet".to_string());

        let body_desc = RigidBodyDesc::new()
            .translation(position)
            .velocity(nphysics2d::math::Velocity::<f32>::new(velocity, 0.0))
            .mass(1000.0)
            .kinematic_rotation(false)
            .name("bullet".to_string());

        let mut body_component_desc = components::PhysicsBodyComponentDesc::new(body_desc);
        body_component_desc.add_collider(collider_desc);

        body_component_desc
    };

    let entity_prototype = EntityPrototype::new(vec![
        Box::new(CloneComponentPrototype::new(
            components::PositionComponent::new(position),
        )),
        Box::new(CloneComponentPrototype::new(
            components::VelocityComponent::new(velocity),
        )),
        Box::new(PhysicsBodyComponentPrototype::new(body_component_desc)),
        Box::new(CloneComponentPrototype::new(
            components::DebugDrawCircleComponent::new(radius, color),
        )),
        Box::new(CloneComponentPrototype::new(
            components::BulletComponent::new(),
        )),
        Box::new(CloneComponentPrototype::new(
            components::FreeAtTimeComponent::new(time_state.frame_start_instant + lifetime),
        )),
    ]);
    entity_factory.enqueue_create(entity_prototype);
}
