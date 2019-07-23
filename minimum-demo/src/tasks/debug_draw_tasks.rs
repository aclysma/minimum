
use minimum::systems::{DataRequirement, Read, ReadOption, async_dispatch::Task, Write};
use minimum::Component;
use minimum::ComponentStorage;

use crate::components;
use crate::resources::DebugDraw;


pub struct UpdateDebugDraw;
impl Task for UpdateDebugDraw {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        Read<<components::DebugDrawCircleComponent as minimum::component::Component>::Storage>,
        Read<<components::PositionComponent as minimum::component::Component>::Storage>,
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
