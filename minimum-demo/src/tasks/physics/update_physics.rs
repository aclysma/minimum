use minimum::resource::{DataRequirement, Read, Write};

use framework::resources::TimeState;
use crate::resources::PhysicsManager;

use minimum::{ResourceTaskImpl, TaskConfig, ResourceTask};
use named_type::NamedType;

#[derive(NamedType)]
pub struct UpdatePhysics;
pub type UpdatePhysicsTask = ResourceTask<UpdatePhysics>;
impl ResourceTaskImpl for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);
    //const REQUIRED_FLAGS: usize = framework::context_flags::PLAYMODE_PLAYING as usize;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePhysics>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}
