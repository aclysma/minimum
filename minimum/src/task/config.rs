use std::prelude::v1::*;

use super::ResourceId;
use super::Task;
use super::RegisteredType;
use super::TaskFactory;
use super::Phase;
use super::TaskContextFlagsFilter;

/// Used internally to list all of a task's requirements
/// Passed into a task's "configure" function to accumulate their settings
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TaskConfig {
    pub(super) read_all: bool,
    pub(super) write_all: bool,
    pub(super) reads: Vec<ResourceId>,
    pub(super) writes: Vec<ResourceId>,
    pub(super) require_run_before: Vec<RegisteredType>,
    pub(super) require_run_after: Vec<RegisteredType>,
    pub(super) require_run_during: Vec<RegisteredType>,
    pub(super) context_flags_filter: TaskContextFlagsFilter,
    #[derivative(Debug="ignore")]
    pub(super) task: Option<Box<dyn Task>>
}

impl TaskConfig {
    pub fn new(task: Option<Box<dyn Task>>) -> Self {
        TaskConfig {
            read_all: false,
            write_all: false,
            reads: vec![],
            writes: vec![],
            require_run_before: vec![],
            require_run_after: vec![],
            require_run_during: vec![],
            context_flags_filter: TaskContextFlagsFilter::default(),
            task
        }
    }

    /// Add a resource to which we must acquire read access
    pub fn add_read(&mut self, resource_id: ResourceId) {
        self.reads.push(resource_id);
    }

    /// Add a resource to which we must acquire write access
    pub fn add_write(&mut self, resource_id: ResourceId) {
        self.writes.push(resource_id);
    }

    /// Indicate that this task will need read access to all resources
    pub fn read_all(&mut self) { self.read_all = true; }

    /// Indicate that this task will need write access to all resources
    pub fn write_all(&mut self) { self.write_all = true; }

//    /// Ensure that the current task will run before T
//    pub fn this_runs_before_task<T: TaskFactory>(&mut self) {
//        self.require_run_after.push(RegisteredType::of::<T>());
//    }
//
//    /// Ensure that the current task will run after T
//    pub fn this_runs_after_task<T: TaskFactory>(&mut self) {
//        self.require_run_before.push(RegisteredType::of::<T>());
//    }

    pub fn this_provides_data_to<T: TaskFactory>(&mut self) {
        self.require_run_after.push(RegisteredType::of::<T>());
    }

    pub fn this_uses_data_from<T : TaskFactory>(&mut self) {
        self.require_run_before.push(RegisteredType::of::<T>());
    }

    /// Ensure that the current task will run before T
    pub fn this_runs_before_phase<T: Phase>(&mut self) {
        self.require_run_after.push(RegisteredType::of::<T>());
    }

    /// Ensure that the current task will run after T
    pub fn this_runs_after_phase<T: Phase>(&mut self) {
        self.require_run_before.push(RegisteredType::of::<T>());
    }

    /// Ensure that the current task will run during T (more specifically, before T, and after T's
    /// direct dependencies
    pub fn this_runs_during_phase<T: Phase>(&mut self) {
        self.require_run_during.push(RegisteredType::of::<T>());
    }

    pub fn run_only_if(&mut self, required_flags: usize) {
        self.context_flags_filter.run_only_if(required_flags);
    }

    pub fn skip_if(&mut self, required_flags: usize) {
        self.context_flags_filter.skip_if(required_flags);
    }
}
