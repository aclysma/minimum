//
// External Requirements
//
use crate::resource::DataRequirement;
use crate::resource::ResourceId;
use crate::ResourceMap;
use crate::util::TrustCell;

//
// Everything we export
//
mod config;
pub use config::TaskConfig;

mod registered_type;
pub use registered_type::RegisteredType;

mod require_resources;
pub use require_resources::RequiresResources;
pub use require_resources::RequiredResources;

mod traits;
pub use traits::Task;
pub use traits::Phase;
pub use traits::TaskFactory;

mod stage;
use stage::TaskStage;

mod dependency_list;
pub use dependency_list::TaskDependencyListBuilder;
pub use dependency_list::TaskDependencyList;

mod tasks;
pub use tasks::read_all_task::ReadAllTask;
pub use tasks::read_all_task::ReadAllTaskImpl;
pub use tasks::write_all_task::WriteAllTask;
pub use tasks::write_all_task::WriteAllTaskImpl;
pub use tasks::resource_task::ResourceTask;
pub use tasks::resource_task::ResourceTaskImpl;

mod context_flags;
pub use context_flags::TaskContextFlags;
pub use context_flags::TaskContextFlagsFilter;
pub use context_flags::TaskWithFilter;

mod schedulers;
pub use schedulers::TaskScheduleBuilderSingleThread;
pub use schedulers::TaskScheduleSingleThread;
pub use schedulers::TaskScheduleBuilderMultiThread;
pub use schedulers::TaskScheduleMultiThread;

mod dispatch_control;
pub use dispatch_control::DispatchControl;

//TODO: Move default phases to framework

pub struct PhaseFrameBegin;
impl Phase for PhaseFrameBegin {
    fn configure(_config: &mut TaskConfig) {

    }
}

pub struct PhaseGatherInput;
impl Phase for PhaseGatherInput {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhaseFrameBegin>();
    }
}

pub struct PhasePrePhysicsGameplay;
impl Phase for PhasePrePhysicsGameplay {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhaseGatherInput>();

    }
}

pub struct PhasePhysics;
impl Phase for PhasePhysics {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhasePrePhysicsGameplay>();
    }
}

pub struct PhasePostPhysicsGameplay;
impl Phase for PhasePostPhysicsGameplay {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhasePhysics>();

    }
}

pub struct PhasePreRender;
impl Phase for PhasePreRender {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhasePostPhysicsGameplay>();

    }
}

pub struct PhaseRender;
impl Phase for PhaseRender {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhasePreRender>();

    }
}

pub struct PhasePostRender;
impl Phase for PhasePostRender {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhaseRender>();

    }
}

pub struct PhaseEndFrame;
impl Phase for PhaseEndFrame {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhasePostRender>();

    }
}
