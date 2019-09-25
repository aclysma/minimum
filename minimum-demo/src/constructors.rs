use crate::components;
use framework::{CloneComponentPrototype, FrameworkEntityPersistencePolicy};
use framework::components::transform;
use framework::FrameworkEntityPrototype;

use framework::components::TransformComponent;
use framework::components::VelocityComponent;

use components::PhysicsBodyComponentPrototypeBox;
use components::PhysicsBodyComponentPrototypeCircle;
use components::PhysicsBodyComponentPrototypeCustom;
use rand::Rng;

const COLLISION_GROUP_PLAYER_INDEX: u32 = 0;
const COLLISION_GROUP_BULLETS_INDEX: u32 = 1;
const COLLISION_GROUP_WALL_INDEX: u32 = 2;

const COLLISION_GROUP_PLAYER_MASK: u32 = 1 << COLLISION_GROUP_PLAYER_INDEX;
const COLLISION_GROUP_BULLETS_MASK: u32 = 1 << COLLISION_GROUP_BULLETS_INDEX;
const COLLISION_GROUP_WALL_MASK: u32 = 1 << COLLISION_GROUP_WALL_INDEX;

const COLLISION_GROUP_ALL_MASK: u32 =
    COLLISION_GROUP_PLAYER_MASK | COLLISION_GROUP_BULLETS_MASK | COLLISION_GROUP_WALL_MASK;

// Left as an example, but this is being loaded from file
pub fn create_wall(
    center: glm::Vec2,
    size: glm::Vec2,
    entity_factory: &mut minimum::EntityFactory,
) {
    #[cfg(feature = "dim3")]
    let center = glm::vec2_to_vec3(&center);
    #[cfg(feature = "dim3")]
    let size = glm::vec2_to_vec3(&size);

    let color = glm::Vec4::new(0.0, 1.0, 1.0, 1.0);
    let mass = 0.0;

    let pec = FrameworkEntityPrototype::new(
        std::path::PathBuf::from("testpath"),
        FrameworkEntityPersistencePolicy::Persistent,
        vec![
            Box::new(CloneComponentPrototype::new(
                TransformComponent::new(center, transform::default_scale(), transform::default_rotation()),
            )),
            Box::new(CloneComponentPrototype::new(
                framework::components::DebugDrawRectComponent::new(size, color),
            )),
            Box::new(PhysicsBodyComponentPrototypeBox::new(
                size,
                mass,
                COLLISION_GROUP_WALL_MASK,
                COLLISION_GROUP_ALL_MASK,
                0,
            )),
        ],
    );

    entity_factory.enqueue_create(Box::new(pec));
}

// Left as an example, but this is being loaded from file
pub fn create_player(entity_factory: &mut minimum::EntityFactory) {
    let position = glm::zero();
    let radius = 15.0;
    let color = glm::Vec4::new(0.0, 1.0, 0.0, 1.0);
    let mass = 1000.0;

    let entity_prototype = FrameworkEntityPrototype::new(
        std::path::PathBuf::from("player"),
        FrameworkEntityPersistencePolicy::Persistent,
        vec![
            Box::new(CloneComponentPrototype::new(
                components::PlayerComponent::new(),
            )),
            Box::new(CloneComponentPrototype::new(
                TransformComponent::new(position, transform::default_scale(), transform::default_rotation()),
            )),
            Box::new(CloneComponentPrototype::new(
                framework::components::DebugDrawCircleComponent::new(radius, color),
            )),
            Box::new(PhysicsBodyComponentPrototypeCircle::new(
                radius,
                mass,
                COLLISION_GROUP_PLAYER_MASK,
                COLLISION_GROUP_WALL_MASK,
                0,
            )),
        ],
    );
    entity_factory.enqueue_create(Box::new(entity_prototype));
}

pub fn create_bullet(
    position: glm::Vec2,
    velocity: glm::Vec2,
    time_state: &framework::resources::TimeState,
    entity_factory: &mut minimum::EntityFactory,
) {
    #[cfg(feature = "dim3")]
    let position = glm::vec2_to_vec3(&position);
    #[cfg(feature = "dim3")]
    let velocity = glm::vec2_to_vec3(&velocity);

    let radius = 5.0;
    let color = glm::Vec4::new(1.0, 1.0, 0.0, 1.0);
    let lifetime = std::time::Duration::from_secs(4);
    let mut rng = rand::thread_rng();

    let restitution = rng.gen_range(0.8, 1.0);

    use ncollide::shape::{Ball, ShapeHandle};
    let shape = ShapeHandle::new(Ball::new(radius));

    let body_component_desc = {
        use nphysics::material::{BasicMaterial, MaterialHandle};
        use nphysics::object::{ColliderDesc, RigidBodyDesc};

        let collider_desc = ColliderDesc::new(shape.clone())
            .material(MaterialHandle::new(BasicMaterial::new(restitution, 0.0)))
            .collision_groups(
                ncollide::world::CollisionGroups::new()
                    .with_membership(&[COLLISION_GROUP_BULLETS_INDEX as usize])
                    .with_blacklist(&[COLLISION_GROUP_BULLETS_INDEX as usize]),
            )
            .name("bullet".to_string());

        let body_desc = RigidBodyDesc::new()
            .translation(position)
            .velocity(nphysics::math::Velocity::<f32>::new(velocity, glm::zero()))
            .mass(1000.0)
            .name("bullet".to_string());

        #[cfg(feature = "dim2")]
        let body_desc = body_desc.kinematic_rotation(false);
        #[cfg(feature = "dim3")]
        let body_desc = body_desc.kinematic_rotations(nphysics::math::Vector::repeat(false));

        let mut body_component_desc = components::PhysicsBodyComponentDesc::new(body_desc);
        body_component_desc.add_collider(collider_desc);

        body_component_desc
    };

    let entity_prototype = FrameworkEntityPrototype::new(
        std::path::PathBuf::from("bullet"),
        FrameworkEntityPersistencePolicy::Transient,
        vec![
            Box::new(CloneComponentPrototype::new(
                TransformComponent::new(position, transform::default_scale(), transform::default_rotation()),
            )),
            Box::new(CloneComponentPrototype::new(
                VelocityComponent::new(velocity),
            )),
            Box::new(PhysicsBodyComponentPrototypeCustom::new(
                body_component_desc,
            )),
            Box::new(CloneComponentPrototype::new(
                framework::components::DebugDrawCircleComponent::new(radius, color),
            )),
            Box::new(CloneComponentPrototype::new(
                components::BulletComponent::new(),
            )),
            Box::new(CloneComponentPrototype::new(
                framework::components::FreeAtTimeComponent::new(
                    time_state.playing().frame_start_instant + lifetime,
                ),
            )),
        ],
    );
    entity_factory.enqueue_create(Box::new(entity_prototype));
}
