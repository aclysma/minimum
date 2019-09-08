use minimum::component::ComponentStorage;
use minimum::resource::{DataRequirement, Read, Write};
use minimum::{EntityHandle, ReadComponent, ResourceTaskImpl, TaskConfig};

use crate::components;
use crate::resources::DebugDraw;
#[cfg(feature = "editor")]
use framework::resources::editor::EditorCollisionWorld;
use named_type::NamedType;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::object::ColliderHandle;

#[derive(NamedType)]
pub struct EditorDrawSelectionShapes;
pub type EditorDrawSelectionShapesTask = minimum::ResourceTask<EditorDrawSelectionShapes>;
impl ResourceTaskImpl for EditorDrawSelectionShapes {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<EditorCollisionWorld>,
        ReadComponent<framework::components::editor::EditorShapeComponent>,
        ReadComponent<framework::components::editor::EditorSelectedComponent>,
    );
    //const REQUIRED_FLAGS: usize =
    //    framework::context_flags::AUTHORITY_CLIENT as usize | framework::context_flags::PLAYMODE_SYSTEM;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.this_uses_data_from::<crate::tasks::editor::EditorHandleInputTask>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
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

            //let mut color = glm::vec4(1.0, 0.0, 0.0, 0.0);a
            if selected_components.exists(co.data()) {
                let color = glm::vec4(1.0, 1.0, 1.0, 1.0);
                let aabb = co.shape().aabb(co.position());

                let expand = glm::vec2(2.0, 2.0);

                debug_draw.add_rect(
                    glm::vec2(aabb.mins().x, aabb.mins().y) - expand,
                    glm::vec2(aabb.maxs().x, aabb.maxs().y) + expand,
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
}
