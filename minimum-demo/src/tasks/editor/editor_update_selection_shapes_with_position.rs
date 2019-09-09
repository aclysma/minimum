use minimum::resource::{DataRequirement, Read, Write};

#[cfg(feature = "editor")]
use framework::resources::editor::EditorCollisionWorld;

use crate::components;
use minimum::component::ReadComponent;
use minimum::{ComponentStorage, EntitySet, ResourceTaskImpl, TaskConfig};

pub struct EditorUpdateSelectionShapesWithPosition;
pub type EditorUpdateSelectionShapesWithPositionTask = minimum::ResourceTask<EditorUpdateSelectionShapesWithPosition>;
impl ResourceTaskImpl for EditorUpdateSelectionShapesWithPosition {
    type RequiredResources = (
        Read<EntitySet>,
        Write<EditorCollisionWorld>,
        ReadComponent<framework::components::editor::EditorShapeComponent>,
        ReadComponent<components::PositionComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
        config.this_provides_data_to::<crate::tasks::editor::EditorUpdateSelectionWorldTask>();
    }

    //const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_SYSTEM as usize;

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
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
