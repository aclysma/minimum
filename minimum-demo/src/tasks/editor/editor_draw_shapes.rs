use minimum::component::ComponentStorage;
use minimum::resource::{DataRequirement, Read, Write};
use minimum::{EntityHandle, ReadComponent, Task, TaskContext};

use crate::components;
use crate::resources::DebugDraw;
use crate::resources::EditorCollisionWorld;
use named_type::NamedType;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::object::ColliderHandle;

#[derive(NamedType)]
pub struct EditorDrawShapes;
impl Task for EditorDrawShapes {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<EditorCollisionWorld>,
        ReadComponent<components::EditorShapeComponent>,
        ReadComponent<components::EditorSelectedComponent>,
    );
    const REQUIRED_FLAGS: usize =
        crate::context_flags::AUTHORITY_CLIENT as usize | crate::context_flags::PLAYMODE_SYSTEM;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (mut debug_draw, editor_collision_world, editor_shape_components, selected_components) =
            data;

        for component in editor_shape_components.iter_values() {
            let _collider_handle: &ColliderHandle = component.collider_handle();
            let _shape_handle: &ShapeHandle<f32> = component.shape_handle();

            //TODO: Draw actual shapes instead of just aabb
        }

        for collision_object in editor_collision_world.world().collision_objects() {
            let co: &ncollide2d::world::CollisionObject<f32, EntityHandle> = collision_object;
            //println!("found a co {:?}", co.position());

            let mut color = glm::vec4(1.0, 0.0, 0.0, 1.0);
            if selected_components.exists(co.data()) {
                color = glm::vec4(1.0, 1.0, 1.0, 1.0);
            }

            let aabb = co.shape().aabb(co.position());
            debug_draw.add_rect(
                glm::vec2(aabb.mins().x, aabb.mins().y),
                glm::vec2(aabb.maxs().x, aabb.maxs().y),
                color,
            );

            debug_draw.add_circle(
                glm::vec2(co.position().translation.x, co.position().translation.y),
                5.0,
                color,
            );
        }
    }
}
