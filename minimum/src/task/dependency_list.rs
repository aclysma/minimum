use hashbrown::HashMap;

use super::RegisteredType;
use super::TaskConfig;
use super::TaskFactory;
use super::Phase;

// Register tasks in the TaskScheduleBuilder
// Process unordered tasks to produce depenency-aware ordering
//  - detect cycles if they exist and fail
// Process ordered tasks to produce a multi-thread friendly execution path
//

/// Used to construct a task schedule
pub struct TaskDependencyListBuilder {
    tasks: HashMap<RegisteredType, TaskConfig>
}

impl TaskDependencyListBuilder {
    /// Create an empty schedule
    pub fn new() -> Self {
        TaskDependencyListBuilder {
            tasks: HashMap::default()
        }
    }

    /// Add a task to be scheduled when `build()` is called
    pub fn add_task_factory<T : TaskFactory>(&mut self) {
        let mut task_config = TaskConfig::new(Some(T::create()));
        let registered_type = RegisteredType::of::<T>();
        T::configure(&mut task_config);

        //println!("{:?}: {:?}", registered_type, task_config);
        self.tasks.insert(registered_type, task_config);
    }

    /// Add a phase to be scheduled when `build()` is called
    pub fn add_phase<T : Phase>(&mut self) {
        let mut task_config = TaskConfig::new(None);
        let registered_type = RegisteredType::of::<T>();
        T::configure(&mut task_config);

        //println!("{:?}: {:?}", registered_type, task_config);
        self.tasks.insert(registered_type, task_config);
    }

    /// Examine all configuration for the task factories/phases that were added and try to produce
    /// an update ordering that satisfies all requirements
    pub fn build(mut self) -> TaskDependencyList {
        // Contains dependences from before/after rules
        // i.e. Task A depends on tasks X, Y, Z finishing
        let mut before_after_dependencies = HashMap::<RegisteredType, Vec<RegisteredType>>::new();

        // Populate before_after_dependencies with require_run_before requirements
        for (task, config) in &self.tasks {
            let entry = before_after_dependencies.entry(task.clone()).or_insert(vec![]);
            for before_task in &config.require_run_before {
                // task depends on before_task finishing
                entry.push(before_task.clone());
            }
        }

        // Populate before_after_dependencies with require_run_after requirements
        for (task, config) in &self.tasks {
            for after_task in &config.require_run_after {
                // task depends on after_task finishing
                before_after_dependencies.entry(after_task.clone()).or_insert(vec![]).push(task.clone());
            }
        }

        // Contains dependencies from during rules
        // Imagine Phase A executes before Phase B. Tasks that execute during B depend on all of B's
        // dependencies (A), and B will depend on tasks that execute during B. This is kind of like
        // splicing the requirements into a linked list.
        // Like above, map reads like: Task A depends on tasks X, Y, Z finishing
        let mut during_dependencies = HashMap::<RegisteredType, Vec<RegisteredType>>::new();
        for (task, config) in &self.tasks {
            for during_phase in &config.require_run_during {
                // task depends on during_phase's dependencies (phase_dependencies) AS THEY WERE BEFORE THIS LOOP
                // This is why we keep before_after_dependencies and during_dependencies separate for now
                let phase_dependencies = before_after_dependencies.entry(during_phase.clone()).or_insert(vec![]);
                for phase_dependency in phase_dependencies {
                    during_dependencies.entry(task.clone()).or_insert(vec![]).push(phase_dependency.clone());
                }

                // during_phase depends on the task that executes within it
                during_dependencies.entry(during_phase.clone()).or_insert(vec![]).push(task.clone());
            }
        }

        // Make sure every task is in the hashmap, even if it has no dependencies
        let mut combined_dependencies = before_after_dependencies;
        for (task, config) in &self.tasks {
            combined_dependencies.entry(task.clone()).or_insert(vec![]);
        }

        // Merge the during_dependencies into the combined_dependencies
        for (task, dependencies) in during_dependencies {
            let entry = combined_dependencies.entry(task.clone()).or_insert(vec![]);
            for dependency in dependencies {
                entry.push(dependency.clone());
            }
        }

        // Now, simulate draining the tasks.
        let mut execution_order = vec![];
        while !combined_dependencies.is_empty() {
            let mut ready_tasks = vec![];

            // Find all tasks that have no dependencies. We assume they will complete immediately
            for (task, dependencies) in &combined_dependencies {
                if dependencies.is_empty() {
                    ready_tasks.push(task.clone());
                }
            }

            // Check that something cleared. If nothing cleared, we have a cycle/deadlock.
            if ready_tasks.is_empty() {
                // Cycle detected

                let first_key;
                for (key, value) in &combined_dependencies {
                    first_key = key;

                    let mut key = first_key;
                    let mut cycle = vec![first_key.clone()];
                    loop {
                        let values = &combined_dependencies[key];
                        cycle.push(values[0].clone());
                        key = &values[0];

                        if cycle[0] == cycle[cycle.len() - 1] {
                            for i in 0..cycle.len() - 1 {
                                println!("{:?} depends on {:?}", cycle[i], cycle[i + 1]);
                            }

                            panic!("Could not produce schedule, a task dependency cycle was detected");
                        }
                    }
                };
            }

            // Remove each ready task from the hashmap (both keys/values)
            for ready_task in &ready_tasks {
                // Remove the task's key
                combined_dependencies.remove(&ready_task);

                // Remove the task from all other tasks's values, wherever it exists
                for (task, mut dependencies) in combined_dependencies.iter_mut() {
                    dependencies.iter().position(|x| *x == *ready_task).map(|i| dependencies.swap_remove(i));
                }
            }

            // Append the tasks that are ready to the execution order
            for task in ready_tasks {
                let task_config = self.tasks.remove(&task).unwrap();

                // Don't push phases.. they are only used for creating the dependency list
                //TODO: Should push Tasks that aren't Option
                if task_config.task.is_some() {
                    execution_order.push(task_config);
                }
            }
        }

        assert!(self.tasks.is_empty());


        for task in &execution_order {
            println!("task: {:?}", task);
        }

        //TODO: Produce a future or a vector of callbacks that can be run
        //TODO: Support for flags to turn things on/off
        //TODO: T param for global param that gets passed in to run()

        let schedule = TaskDependencyList::new(execution_order);
        schedule
    }
}

/// A calculated task schedule. Can be used to call all tasks in the schedule
pub struct TaskDependencyList {
    pub execution_order: Vec<TaskConfig>
}

impl TaskDependencyList {
    pub fn new(execution_order: Vec<TaskConfig>) -> Self {
        TaskDependencyList {
            execution_order
        }
    }
}


