use minimum::resource::{DataRequirement, Read};

use crate::resources::PhysicsManager;

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::{ComponentStorage, EntitySet, ResourceTaskImpl, TaskConfig, ResourceTask};

pub struct PhysicsSyncPost;
pub type PhysicsSyncPostTask = ResourceTask<PhysicsSyncPost>;
impl ResourceTaskImpl for PhysicsSyncPost {
    type RequiredResources = (
        Read<EntitySet>,
        Read<PhysicsManager>,
        ReadComponent<components::PhysicsBodyComponent>,
        WriteComponent<components::PositionComponent>,
        WriteComponent<components::VelocityComponent>,
    );
    //const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_PLAYING as usize;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePhysics>();
        config.this_uses_data_from::<crate::tasks::UpdatePhysicsTask>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
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
            let body: &nphysics2d::object::RigidBody<f32> = physics_manager
                .world()
                .rigid_body(body_component.body_handle())
                .unwrap();

            if let Some(pos_component) = pos_components.get_mut(&entity) {
                *pos_component.position_mut() = body.position().translation.vector;
            }

            if let Some(vel_component) = vel_components.get_mut(&entity) {
                *vel_component.velocity_mut() = body.velocity().linear;
            }
        }
    }
}
