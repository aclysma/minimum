use std::prelude::v1::*;

use super::TaskConfig;
use super::TaskDependencyList;
use super::TrustCell;
use super::ResourceMap;
use super::TaskContextFlags;
use super::TaskWithFilter;

pub struct TaskScheduleBuilderSingleThread {
    execution_order: Vec<TaskConfig>
}

impl TaskScheduleBuilderSingleThread {
    pub fn new(execution_order: TaskDependencyList) -> Self {
        TaskScheduleBuilderSingleThread {
            execution_order: execution_order.execution_order
        }
    }

    pub fn build(self) -> TaskScheduleSingleThread {
        let mut tasks = vec![];
        for task_config in self.execution_order {
            tasks.push(TaskWithFilter::new(task_config));
        }

        TaskScheduleSingleThread::new(tasks)
    }
}

pub struct TaskScheduleSingleThread {
    tasks: Vec<TaskWithFilter>
}

impl TaskScheduleSingleThread {
    pub fn new(tasks: Vec<TaskWithFilter>) -> Self {
        TaskScheduleSingleThread {
            tasks
        }
    }

    pub fn step(&self, context_flags: &TaskContextFlags, resource_map: &TrustCell<ResourceMap>) {
        for task in &self.tasks {
            task.run_if_filter_passes(context_flags, resource_map);
        }
    }
}

