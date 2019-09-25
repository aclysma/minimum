use minimum::resource::{DataRequirement, Read, Write};

use crate::resources::PhysicsManager;

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::{
    ComponentStorage, EntitySet, ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags,
};

pub struct PhysicsSyncPre;
pub type PhysicsSyncPreTask = ResourceTask<PhysicsSyncPre>;
impl ResourceTaskImpl for PhysicsSyncPre {
    type RequiredResources = (
        Read<EntitySet>,
        Write<PhysicsManager>,
        ReadComponent<components::PhysicsBodyComponent>,
        WriteComponent<framework::components::TransformComponent>,
        WriteComponent<framework::components::VelocityComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePhysics>();
        config.this_provides_data_to::<crate::tasks::UpdatePhysicsTask>();
        config.run_only_if(framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            mut physics_manager,
            physics_body_components,
            mut pos_components,
            mut vel_components,
        ) = data;

        for (entity, body_component) in physics_body_components.iter(&entity_set) {
            let body: &mut nphysics::object::RigidBody<f32> = physics_manager
                .world_mut()
                .rigid_body_mut(body_component.body_handle())
                .unwrap();

            if let Some(pos_component) = pos_components.get_mut(&entity) {
                if pos_component.requires_sync_to_physics() {
                    body.set_position(nphysics::math::Isometry::from_parts(
                        nphysics::math::Translation::from(pos_component.position()),
                        body.position().rotation,
                    ));

                    pos_component.clear_requires_sync_to_physics();
                }
            }

            if let Some(vel_component) = vel_components.get_mut(&entity) {
                if vel_component.requires_sync_to_physics() {
                    body.set_linear_velocity(vel_component.velocity());
                    vel_component.clear_requires_sync_to_physics();
                }
            }
        }
    }
}
