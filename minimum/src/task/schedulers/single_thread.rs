use std::prelude::v1::*;

use super::TaskConfig;
use super::TaskDependencyList;
use super::Task;
use super::TrustCell;
use super::ResourceMap;

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
            tasks.push(task_config.task.unwrap());
        }

        TaskScheduleSingleThread::new(tasks)
    }
}

pub struct TaskScheduleSingleThread {
    tasks: Vec<Box<dyn Task>>
}

impl TaskScheduleSingleThread {
    pub fn new(tasks: Vec<Box<dyn Task>>) -> Self {
        TaskScheduleSingleThread {
            tasks
        }
    }

    pub fn step(&self, resource_map: &TrustCell<ResourceMap>) {
        for task in &self.tasks {
            task.run(resource_map);
        }
    }
}

