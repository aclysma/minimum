
use super::TaskConfig;
use super::TaskDependencyList;
use super::TaskStage;
use super::Task;
use super::TrustCell;
use super::ResourceMap;

//TODO: All of this, async_dispatcher will need to be refactored to work here and the old dispatcher removed
/*

pub struct TaskScheduleBuilderFutureGraph {
    execution_order: Vec<TaskConfig>
}

impl TaskScheduleBuilderFutureGraph {
    pub fn new(execution_order: TaskDependencyList) -> Self {
        TaskScheduleBuilderFutureGraph {
            execution_order: execution_order.execution_order
        }
    }

    pub fn build(self) -> TaskScheduleFutureGraph {
        let mut tasks = vec![];
        for task_config in self.execution_order {
            tasks.push(task_config.task.unwrap());
        }

        TaskScheduleFutureGraph::new(tasks)
    }
}

pub struct TaskScheduleFutureGraph {
    tasks: Vec<Box<dyn Task>>
}

impl TaskScheduleFutureGraph {
    pub fn new(tasks: Vec<Box<dyn Task>>) -> Self {
        TaskScheduleFutureGraph {
            tasks
        }
    }

    pub fn run(&self, resource_map: &TrustCell<ResourceMap>) {
        for task in &self.tasks {
            task.run(resource_map);
        }
    }
}
*/
