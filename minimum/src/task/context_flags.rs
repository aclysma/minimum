use super::TaskConfig;
use super::Task;
use super::TrustCell;
use super::ResourceMap;

/// Passed by ref to all tasks in a single update step
pub struct TaskContextFlags {
    flags: usize
}

impl TaskContextFlags {
    pub fn new(flags: usize) -> Self {
        TaskContextFlags {
            flags
        }
    }

    pub fn flags(&self) -> usize {
        self.flags
    }
}

#[derive(Default, Debug)]
pub struct TaskContextFlagsFilter {
    run_only_if: usize,
    skip_if: usize
}

impl TaskContextFlagsFilter {
    pub fn run_only_if(&mut self, flags: usize) {
        self.run_only_if |= flags;
    }

    pub fn skip_if(&mut self, flags: usize) {
        self.skip_if |= flags;
    }

    pub fn check_filter(&self, context_flags: &TaskContextFlags) -> bool {
        return (self.run_only_if & context_flags.flags) == self.run_only_if &&
            (self.skip_if & context_flags.flags) == 0;
    }
}

pub struct TaskWithFilter {
    task: Box<dyn Task>,
    context_flags_filter: TaskContextFlagsFilter
}

impl TaskWithFilter {
    pub fn new(task_config: TaskConfig) -> Self {
        TaskWithFilter {
            task: task_config.task.unwrap(),
            context_flags_filter: task_config.context_flags_filter
        }
    }

    pub fn run_if_filter_passes(&self, context_flags: &TaskContextFlags, resource_map: &TrustCell<ResourceMap>) {
        if self.context_flags_filter.check_filter(context_flags) {
            self.task.run(context_flags, resource_map);
        }
    }
}