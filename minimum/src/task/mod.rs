//
// External Requirements
//
use named_type::NamedType;
use crate::resource::DataRequirement;
use crate::resource::ResourceId;
use crate::ResourceMap;
use crate::dispatch::async_dispatcher::{RequiresResources, RequiredResources};
use crate::util::TrustCell;

//
// Everything we export
//
mod config;
pub use config::TaskConfig;

mod registered_type;
pub use registered_type::RegisteredType;

mod traits;
pub use traits::Task;
pub use traits::Phase;
pub use traits::TaskFactory;

mod stage;
use stage::TaskStage;

mod dependency_list;
use dependency_list::TaskDependencyListBuilder;
use dependency_list::TaskDependencyList;

mod tasks;
pub use tasks::read_all_task::ReadAllTask;
pub use tasks::read_all_task::ReadAllTaskImpl;
pub use tasks::write_all_task::WriteAllTask;
pub use tasks::write_all_task::WriteAllTaskImpl;
pub use tasks::resource_task::ResourceTask;
pub use tasks::resource_task::ResourceTaskImpl;

mod schedulers;
pub use schedulers::TaskScheduleBuilderSingleThread;
pub use schedulers::TaskScheduleSingleThread;
pub use schedulers::TaskScheduleBuilderMultiThread;
pub use schedulers::TaskScheduleMultiThread;




//TEMPORARY
pub mod proof_of_concept;