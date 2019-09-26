use crate::base::resource::{DataRequirement, Read, Write};

#[cfg(feature = "editor")]
use crate::resources::editor::EditorCollisionWorld;

use crate::base::component::ReadComponent;
use crate::base::{ComponentStorage, EntitySet, ResourceTaskImpl, TaskConfig, TaskContextFlags};

pub struct EditorUpdateSelectionShapesWithPosition;
pub type EditorUpdateSelectionShapesWithPositionTask =
    crate::base::ResourceTask<EditorUpdateSelectionShapesWithPosition>;
impl ResourceTaskImpl for EditorUpdateSelectionShapesWithPosition {
    type RequiredResources = (
        Read<EntitySet>,
        Write<EditorCollisionWorld>,
        ReadComponent<crate::components::editor::EditorShapeComponent>,
        ReadComponent<crate::components::TransformComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePreRender>();
        config.this_provides_data_to::<crate::tasks::editor::EditorUpdateSelectionWorldTask>();
        config.run_only_if(crate::context_flags::PLAYMODE_SYSTEM);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (entity_set, mut collision_world, editor_shape_components, transform_components) = data;

        for (entity, editor_shape_component) in editor_shape_components.iter(&entity_set) {
            if let Some(transform_component) = transform_components.get(&entity) {

                #[cfg(feature = "dim2")]
                let isometry = ncollide::math::Isometry::new(transform_component.position(), transform_component.rotation());

                #[cfg(feature = "dim3")]
                let isometry = ncollide::math::Isometry::from_parts(transform_component.position().into(), UnitQuaternion::from_quaternion(transform_component.rotation()));

                collision_world.world_mut().set_position(
                    *editor_shape_component.collider_handle(),
                    isometry,
                );
            }
        }
    }
}
