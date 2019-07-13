use minimum::systems::{DataRequirement, MinimumDispatcherBuilder, Task, Write};

use minimum::async_dispatcher::ExecuteSequential;

// Keep track of how many frames we've run
struct UpdateCount {
    count: i32,
}

impl UpdateCount {
    fn new() -> Self {
        return UpdateCount { count: 0 };
    }
}

// Mock a physics system
struct PhysicsSystem;
impl PhysicsSystem {
    fn update(&mut self) {}
}

// A task to trigger updating the physics system
struct UpdatePhysicsSystem;
impl Task for UpdatePhysicsSystem {
    type RequiredResources = (Write<PhysicsSystem>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut physics_system = data;
        physics_system.update();
    }
}

fn main() {
    let dispatcher = MinimumDispatcherBuilder::new()
        .with_resource(PhysicsSystem)
        .with_resource(UpdateCount::new())
        .build();

    dispatcher.enter_game_loop(|ctx| {
        ExecuteSequential::new(vec![
            ctx.run_task(UpdatePhysicsSystem),
            Box::new(futures::lazy(move || {
                let world = ctx.world();
                let mut update_count = world.fetch_mut::<UpdateCount>();
                println!("update {}", update_count.count);
                update_count.count += 1;
                if update_count.count > 100 {
                    ctx.end_game_loop();
                }

                Ok(())
            })),
        ])
    });
}
