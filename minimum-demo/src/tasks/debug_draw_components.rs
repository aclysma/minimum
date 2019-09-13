use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, ReadComponent, ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::components;
use crate::resources::DebugDraw;

pub struct DebugDrawComponents;
pub type DebugDrawComponentsTask = minimum::ResourceTask<DebugDrawComponents>;
impl ResourceTaskImpl for DebugDrawComponents {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        ReadComponent<components::DebugDrawCircleComponent>,
        ReadComponent<components::DebugDrawRectComponent>,
        ReadComponent<components::PositionComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.run_only_if(framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (mut debug_draw, entity_set, circle_components, rect_components, position_components) =
            data;

        for (entity_index, circle) in circle_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                debug_draw.add_circle(position.position(), circle.radius(), circle.color())
            }
        }

        for (entity_index, rect) in rect_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                let p0 = position.position() + (rect.size() / 2.0);
                let p1 = position.position() - (rect.size() / 2.0);

                debug_draw.add_rect(p0, p1, rect.color());
            }
        }
    }
}
