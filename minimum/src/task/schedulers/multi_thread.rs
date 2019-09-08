
use super::TaskConfig;
use super::TaskDependencyList;
use super::TaskStage;
use super::TrustCell;
use super::ResourceMap;

pub struct TaskScheduleBuilderMultiThread {
    execution_order: Vec<TaskConfig>
}

impl TaskScheduleBuilderMultiThread {
    pub fn new(execution_order: TaskDependencyList) -> Self {
        TaskScheduleBuilderMultiThread {
            execution_order: execution_order.execution_order
        }
    }

    pub fn build(self) -> TaskScheduleMultiThread {
        let mut all_stages = vec![];
        let mut current_stage = TaskStage::new();

        for task in self.execution_order {
            if current_stage.can_add_task(&task) {
                println!("add to stage");
                current_stage.add_task(task);
            } else {
                println!("push stage");
                all_stages.push(current_stage);
                current_stage = TaskStage::new();
                current_stage.add_task(task);
            }
        }

        if !current_stage.is_empty() {
            println!("push stage");
            all_stages.push(current_stage);
        }

        TaskScheduleMultiThread::new(all_stages)
    }
}

pub struct TaskScheduleMultiThread {
    stages: Vec<TaskStage>
}

impl TaskScheduleMultiThread {
    pub fn new(stages: Vec<TaskStage>) -> Self {
        TaskScheduleMultiThread {
            stages
        }
    }

    pub fn run(&self, resource_map: &TrustCell<ResourceMap>) {
        for stage in &self.stages {
            println!("----------------------");
            //TODO: Use rayon or something to make this actually MT
            for task in stage.tasks() {
                task.run(resource_map);
            }
        }
    }
}


