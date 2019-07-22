extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

mod resources;
mod init;
mod tasks;
mod renderer;

use minimum::systems::async_dispatch::{
    MinimumDispatcher,
    MinimumDispatcherBuilder
};


fn main() {
    run_the_game().unwrap();
}

fn run_the_game() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("vore::level_loader", log::LevelFilter::Warn)
        .filter_module("gfx_backend_metal", log::LevelFilter::Error)
        .filter_module("rendy", log::LevelFilter::Error)
        .init();

    // Any config/data you want to load before opening a window should go here

    let event_loop = winit::event_loop::EventLoop::<resources::WindowUserEvent>::new_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_title("Vore")
        .build(&event_loop)?;

    let mut entity_set = minimum::entity::EntitySet::new();
//    entity_set.register_component_type::<game::TerrainComponent>();
//    entity_set.register_component_type::<game::PickupComponent>();

    let mut world = minimum::systems::WorldBuilder::new()
        .with_resource(resources::GameControl::new())
        .with_resource(resources::DebugDraw::new())
        .with_resource(resources::InputManager::new())
        .with_resource(resources::TimeState::new())
        //.with_resource(gfx::DebugCameraSettings::new())
        //.with_resource(physics::Physics::new())
        .with_resource(window)
        .with_resource(resources::RenderState::empty())
        .with_resource(entity_set)
//        .with_resource(<game::TerrainComponent as minimum::component::Component>::Storage::new())
//        .with_resource(<game::PickupComponent as minimum::component::Component>::Storage::new())
        .build();

    // Assets you want to always have available could be loaded here


    world.insert(init::init_imgui_manager(&world));
    world.insert(init::create_renderer(&world));

    // Wrap the threadsafe interface to the window in WindowInterface and add it to the world
    // Return the event_tx which needs to be given to the event loop
    let winit_event_tx = init::create_window_interface(&mut world, &event_loop); //TODO: continue moving things to init.rs

    // Start the game loop thread
    let dispatcher = MinimumDispatcherBuilder::from_world(world).build();
    let _join_handle = std::thread::spawn(|| dispatcher_thread(dispatcher));

    // Hand control of the main thread to winit
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::UserEvent(resources::WindowUserEvent::Terminate) => {
            *control_flow = winit::event_loop::ControlFlow::Exit
        }
        _ => {
            winit_event_tx.send(event).unwrap();
        }
    });

    //NOTE: The game terminates when the event_loop halts, so any code here onwards won't execute
}

//TODO: Can this all be visualized as a graph?

fn dispatcher_thread(dispatcher: MinimumDispatcher) -> minimum::systems::World {
    info!("dispatch thread started");

    let mut world = dispatcher.enter_game_loop(move |dispatch_ctx| {

        let dispatch_context1 = dispatch_ctx.clone();
        let dispatch_context2 = dispatch_ctx.clone();

        minimum::async_dispatcher::ExecuteSequential::new(vec![
            dispatch_ctx.run_task(tasks::HandleInput),
            //dispatch_ctx.run_task(tasks::UpdateRenderer), //TODO: Is this really necessary to have?
            dispatch_ctx.run_task(tasks::ImguiBeginFrame),
            dispatch_ctx.run_task(tasks::RenderImguiMainMenu),
            //dispatch_ctx.run_task(tasks::UpdateDebugCameraSettings),

            // Conditionally execute tasks
            // Way to pause the game and get mutable access to the world
//            Box::new(futures::lazy(move || {
//                if !dispatch_context1.has_resource::<game::GameState>() {
//                    return minimum::async_dispatcher::ExecuteSequential::new(vec![]);
//                }
//
//                minimum::async_dispatcher::ExecuteSequential::new(vec![
//                    dispatch_context1.run_task(tasks::PrePhysics),
//                    dispatch_context1.run_task(tasks::Physics),
//                    dispatch_context1.run_task(tasks::PostPhysics),
//                    dispatch_context1.run_task(tasks::DebugDraw),
//                ])
//            })),

//            dispatch_ctx.run_task(tasks::UpdateFileLoader),
//            dispatch_ctx.run_task(tasks::UpdateTextureCache),

            dispatch_ctx.visit_world(|world| {
                tasks::render(world);
            }),

            dispatch_ctx.run_task(tasks::UpdateTimeState),
            dispatch_ctx.run_task(tasks::ClearDebugDraw),

            dispatch_ctx.visit_world_mut(move |world| {
                let mut entity_set = world.fetch_mut::<minimum::entity::EntitySet>();
                entity_set.flush_free(world);

                let mut game_control = world.fetch_mut::<resources::GameControl>();

                if game_control.terminate_process() {
                    dispatch_context2.end_game_loop();
                } else if game_control.has_load_level() {

                    // Unload game state
                    entity_set.clear(world);
                    //world.remove::<game::GameState>();

                    // Setup game state
                    let level_to_load = game_control.take_load_level();
                    //world.insert::<physics::Physics>();

                }
            }),
        ])
    });

    // This would be a good spot to flush anything out like saved progress

    // Manual dispose is required for rendy
    let renderer = world.remove::<renderer::Renderer>();
    renderer.unwrap().dispose(&world);

    world
        .fetch_mut::<resources::WindowInterface>()
        .event_loop_proxy
        .send_event(resources::WindowUserEvent::Terminate)
        .unwrap();

    info!("dispatch thread is done");
    world
}
