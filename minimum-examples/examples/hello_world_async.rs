use minimum::systems::{DataRequirement, Write};

use minimum::async_dispatcher::ExecuteSequential;
use minimum::systems::async_dispatch::{MinimumDispatcher, Task};
use minimum::WorldBuilder;

// Mock a physics system
pub struct ExampleResource;
impl ExampleResource {
    pub fn new() -> Self {
        ExampleResource
    }

    pub fn update(&mut self) {
        println!("hello world!");
    }
}

// A task to trigger updating the physics system
pub struct ExampleTask;
impl Task for ExampleTask {
    type RequiredResources = (Write<ExampleResource>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut example_resource = data;
        example_resource.update();
    }
}

fn main() {
    // Set up a dispatcher with the example resource in it
    let world = WorldBuilder::new()
        .with_resource(ExampleResource::new())
        .build();

    let dispatcher = MinimumDispatcher::new(world);

    // Start the game loop
    dispatcher.enter_game_loop(|ctx| {
        ExecuteSequential::new(vec![
            // Run a task, this will call update on the given resource
            ctx.run_task(ExampleTask),
            Box::new(futures::lazy(move || {
                ctx.end_game_loop();
                Ok(())
            })),
        ])
    });
}
