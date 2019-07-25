use crate::components;
use crate::resources;
use minimum::Component;

//TODO: Probably want a create queue for these to streamline getting the necessary component storages
// and perhaps to allow rate-limiting of construction rate
pub fn create_player(
    entities: &mut minimum::EntitySet,
    position_components: &mut <components::PositionComponent as Component>::Storage,
    debug_draw_components: &mut <components::DebugDrawCircleComponent as Component>::Storage,
    player_components: &mut <components::PlayerComponent as Component>::Storage,
) {
    let entity = entities.allocate_get();

    entity.add_component(&mut *player_components, components::PlayerComponent::new());

    entity.add_component(
        &mut *position_components,
        components::PositionComponent::new(glm::zero()),
    );

    entity.add_component(
        &mut *debug_draw_components,
        components::DebugDrawCircleComponent::new(15.0, glm::Vec4::new(0.0, 1.0, 0.0, 1.0)),
    );
}

pub fn create_bullet(
    position: glm::Vec2,
    velocity: glm::Vec2,
    time_state: &resources::TimeState,
    physics_manager: &mut resources::PhysicsManager,
    entities: &mut minimum::EntitySet,
    position_components: &mut <components::PositionComponent as Component>::Storage,
    velocity_components: &mut <components::VelocityComponent as Component>::Storage,
    debug_draw_components: &mut <components::DebugDrawCircleComponent as Component>::Storage,
    bullet_components: &mut <components::BulletComponent as Component>::Storage,
    free_at_time_components: &mut <components::FreeAtTimeComponent as Component>::Storage,
    physics_body_components: &mut <components::PhysicsBodyComponent as Component>::Storage,
) {
    let radius = 5.0;
    let color = glm::Vec4::new(1.0, 0.0, 0.0, 1.0);

    let entity = entities.allocate_get();

    entity.add_component(
        &mut *position_components,
        components::PositionComponent::new(position),
    );

    entity.add_component(
        &mut *velocity_components,
        components::VelocityComponent::new(velocity),
    );

    let body_handle = {
        use ncollide2d::shape::{Ball, ShapeHandle};
        use nphysics2d::material::{BasicMaterial, MaterialHandle};
        use nphysics2d::object::{ColliderDesc, RigidBodyDesc};

        let shape = ShapeHandle::new(Ball::new(radius));
        let collider_desc = ColliderDesc::new(shape)
            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
            .name("bullet".to_string());

        let body = RigidBodyDesc::new()
            .translation(position)
            .mass(1000.0)
            .collider(&collider_desc)
            .kinematic_rotation(false)
            .name("bullet".to_string())
            .build(physics_manager.world_mut());

        body.handle()
    };

    entity.add_component(
        &mut *physics_body_components,
        components::PhysicsBodyComponent::new(body_handle),
    );

    entity.add_component(
        &mut *debug_draw_components,
        components::DebugDrawCircleComponent::new(radius, color),
    );

    entity.add_component(&mut *bullet_components, components::BulletComponent::new());

    entity.add_component(
        &mut *free_at_time_components,
        components::FreeAtTimeComponent::new(
            time_state.frame_start_instant + std::time::Duration::from_secs(4),
        ),
    );
}
