use crate::base::resource::{DataRequirement, Read};

use crate::resources::PhysicsManager;

use crate::components;
use crate::base::component::{ReadComponent, WriteComponent};
use crate::base::{
    ComponentStorage, EntitySet, ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags,
};

pub struct PhysicsSyncPost;
pub type PhysicsSyncPostTask = ResourceTask<PhysicsSyncPost>;
impl ResourceTaskImpl for PhysicsSyncPost {
    type RequiredResources = (
        Read<EntitySet>,
        Read<PhysicsManager>,
        ReadComponent<components::PhysicsBodyComponent>,
        WriteComponent<crate::framework::components::TransformComponent>,
        WriteComponent<crate::framework::components::VelocityComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePhysics>();
        config.this_uses_data_from::<crate::tasks::UpdatePhysicsTask>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            physics_manager,
            physics_body_components,
            mut pos_components,
            mut vel_components,
        ) = data;

        for (entity, body_component) in physics_body_components.iter(&entity_set) {
            let body: &nphysics::object::RigidBody<f32> = physics_manager
                .world()
                .rigid_body(body_component.body_handle())
                .unwrap();

            if let Some(pos_component) = pos_components.get_mut(&entity) {

                let position = body.position().translation.vector;
                #[cfg(feature = "dim2")]
                let position = glm::vec2_to_vec3(&position);

                *pos_component.position_mut() = position;
            }

            if let Some(vel_component) = vel_components.get_mut(&entity) {

                let velocity = body.velocity().linear;
                #[cfg(feature = "dim2")]
                let velocity = glm::vec2_to_vec3(&velocity);

                *vel_component.velocity_mut() = velocity;
            }
        }
    }
}
