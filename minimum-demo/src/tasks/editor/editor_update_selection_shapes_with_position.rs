use minimum::resource::{DataRequirement, Read, Write};

use crate::resources::EditorCollisionWorld;

use crate::components;
use minimum::component::ReadComponent;
use minimum::{ComponentStorage, EntitySet, Task, TaskContext};
use named_type::NamedType;

#[derive(NamedType)]
pub struct EditorUpdateSelectionShapesWithPosition;
impl Task for EditorUpdateSelectionShapesWithPosition {
    type RequiredResources = (
        Read<EntitySet>,
        Write<EditorCollisionWorld>,
        ReadComponent<components::EditorShapeComponent>,
        ReadComponent<components::PositionComponent>,
    );

    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_SYSTEM as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (entity_set, mut collision_world, editor_shape_components, position_components) = data;

        for (entity, editor_shape_component) in editor_shape_components.iter(&entity_set) {
            if let Some(position_component) = position_components.get(&entity) {
                collision_world.world_mut().set_position(
                    *editor_shape_component.collider_handle(),
                    nalgebra::Isometry2::new(position_component.position(), 0.0),
                );
            }
        }
    }
}
