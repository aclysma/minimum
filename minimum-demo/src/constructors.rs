
use minimum::Component;
use crate::components;

pub fn create_player(
    entities: &mut minimum::EntitySet,
    position_components: &mut <components::PositionComponent as Component>::Storage,
    debug_draw_components: &mut <components::DebugDrawCircleComponent as Component>::Storage,
    player_components: &mut <components::PlayerComponent as Component>::Storage
) {
    let entity = entities.allocate_get();

    entity.add_component(
        &mut *player_components,
        components::PlayerComponent::new()
    );

    // Put a position on everything
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
    entities: &mut minimum::EntitySet,
    position_components: &mut <components::PositionComponent as Component>::Storage,
    velocity_components: &mut <components::VelocityComponent as Component>::Storage,
    debug_draw_components: &mut <components::DebugDrawCircleComponent as Component>::Storage,
) {
    let entity = entities.allocate_get();

    // Put a position on everything
    entity.add_component(
        &mut *position_components,
        components::PositionComponent::new(position),
    );

    entity.add_component(
        &mut *velocity_components,
        components::VelocityComponent::new(velocity),
    );

    entity.add_component(
        &mut *debug_draw_components,
        components::DebugDrawCircleComponent::new(5.0, glm::Vec4::new(1.0, 0.0, 0.0, 1.0)),
    );
}