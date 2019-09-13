use minimum::{
    DataRequirement, ResourceTask, ResourceTaskImpl, TaskConfig, TaskContextFlags, WorldBuilder,
    Write,
};

//
// This is an example resource. Resources contain data that tasks can operate on.
//
pub struct ExampleResource;
impl ExampleResource {
    pub fn new() -> Self {
        ExampleResource
    }

    pub fn update(&mut self) {
        println!("hello world!");
    }
}

//
// This is an example task. The dispatcher will run it, passing the resources it requires.
//
pub struct Example;
pub type ExampleTask = ResourceTask<Example>;
impl ResourceTaskImpl for Example {
    type RequiredResources = (Write<ExampleResource>);

    fn configure(_task_config: &mut TaskConfig) {
        // task_config can be used to set up contraints on when this task can run
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let mut example_resource = data;
        example_resource.update();
    }
}

//
// In the main loop, you need to:
//
//   1. Register the resources that will be available
//   2. Start the game loop
//   3. Within the game loop, run tasks
//   4. End the loop
//
// The enter_game_loop call will return all the resources.
//
fn main() {
    // Set up a dispatcher with the example resource in it
    let world = WorldBuilder::new()
        .with_resource(ExampleResource::new())
        .with_task::<ExampleTask>()
        .build_update_loop_single_threaded(0);

    world.step();
}
