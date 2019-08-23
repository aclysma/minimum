use crate::resource;
use resource::ResourceMap;
use std::sync::Arc;

pub struct MinimumDispatcher {
    resource_map: Arc<ResourceMap>,
}

impl MinimumDispatcher {
    pub fn new(resource_map: ResourceMap) -> MinimumDispatcher {
        MinimumDispatcher {
            resource_map: Arc::new(resource_map),
        }
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F>(self, f: F) -> ResourceMap
    where
        F: Fn(&MinimumDispatcherContext),
    {
        {
            let ctx = MinimumDispatcherContext {
                resource_map: self.resource_map.clone(),
                should_terminate: std::cell::Cell::new(false),
            };

            loop {
                (f)(&ctx);
                if ctx.should_terminate.get() {
                    break;
                }
            }
        }

        // Then unwrap the resource_map inside it
        let resource_map = Arc::try_unwrap(self.resource_map).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the resource_map
        resource_map
    }
}

pub struct MinimumDispatcherContext {
    resource_map: Arc<ResourceMap>,
    should_terminate: std::cell::Cell<bool>,
}

//
// Task
//

pub trait Task {
    type RequiredResources: for<'a> crate::resource::DataRequirement<'a> + Send + 'static;

    fn run(&mut self, data: <Self::RequiredResources as crate::resource::DataRequirement>::Borrow);
}

impl MinimumDispatcherContext {
    pub fn end_game_loop(&self) {
        self.should_terminate.set(true);
    }

    pub fn resource_map(&self) -> Arc<ResourceMap> {
        self.resource_map.clone()
    }

    pub fn run_task<T>(&self, mut task: T)
    where
        T: Task,
    {
        use resource::DataRequirement;
        let required_data = <<T as Task>::RequiredResources>::fetch(&self.resource_map);
        task.run(required_data);
    }
}
