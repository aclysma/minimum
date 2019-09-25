use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, ReadComponent, ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::components;
use crate::resources::DebugDraw;
use nalgebra::UnitQuaternion;

pub struct DebugDrawComponents;
pub type DebugDrawComponentsTask = minimum::ResourceTask<DebugDrawComponents>;
impl ResourceTaskImpl for DebugDrawComponents {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        ReadComponent<components::DebugDrawCircleComponent>,
        ReadComponent<components::DebugDrawRectComponent>,
        ReadComponent<components::TransformComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.run_only_if(framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (mut debug_draw, entity_set, circle_components, rect_components, transform_components) =
            data;

        for (entity_index, circle) in circle_components.iter(&entity_set) {
            if let Some(transform) = transform_components.get(&entity_index) {
                debug_draw.add_circle(transform.position().xy(), circle.radius() * transform.uniform_scale(), circle.color())
            }
        }

        for (entity_index, rect) in rect_components.iter(&entity_set) {
            if let Some(transform) = transform_components.get(&entity_index) {
                let rect_size = glm::vec2(rect.size().x * transform.scale().x, rect.size().y * transform.scale().y);
                let half_extents = rect_size / 2.0;
                let p0 = half_extents;
                let p1 = glm::vec2(0.0, 0.0) - half_extents;



                #[cfg(feature = "dim2")]
                let rotation = transform.rotation();

                #[cfg(feature = "dim3")]
                let rotation = UnitQuaternion::from_quaternion(transform.rotation()).euler_angles().2;

                let has_rotation = rotation.abs() > std::f32::MIN_POSITIVE;

                if has_rotation {
                    let mut points = vec![p0, glm::vec2(p0.x, p1.y), p1, glm::vec2(p1.x, p0.y)];
                    for p in &mut points {
                        *p = glm::rotate_vec2(p, rotation) + transform.position().xy();
                    }
                    debug_draw.add_polygon(points, rect.color());

                } else {
                    debug_draw.add_rect(p0 + transform.position().xy(), p1 + transform.position().xy(), rect.color());
                }
            }
        }
    }
}
