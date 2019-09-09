use minimum::{TaskDependencyListBuilder, ResourceTask, ResourceTaskImpl, DataRequirement, Write, TaskConfig, ResourceMapBuilder, TaskScheduleBuilderSingleThread, WorldBuilder};
use named_type::NamedType;

#[macro_use]
extern crate named_type_derive;

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
#[derive(NamedType)]
pub struct Example;
pub type ExampleTask = ResourceTask<Example>;
impl ResourceTaskImpl for Example {
    type RequiredResources = (Write<ExampleResource>);

    fn configure(task_config: &mut TaskConfig) {

    }

    fn run(data: <Self::RequiredResources as DataRequirement>::Borrow) {
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
        .build_update_loop_single_threaded();

    world.step();
}
