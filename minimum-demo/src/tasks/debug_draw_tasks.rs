use minimum::systems::{async_dispatch::Task, DataRequirement, Read, Write};
use minimum::{ComponentStorage, ReadComponent};

use crate::components;
use crate::resources::DebugDraw;

pub struct UpdateDebugDraw;
impl Task for UpdateDebugDraw {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        ReadComponent<components::DebugDrawCircleComponent>,
        ReadComponent<components::PositionComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut debug_draw, entity_set, circle_components, position_components) = data;

        debug_draw.clear();

        for (entity_index, circle) in circle_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                debug_draw.add_circle(position.position(), circle.radius(), circle.color())
            }
        }
    }
}
