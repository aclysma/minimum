extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

mod components;
mod resources;
mod init;
mod tasks;
mod renderer;
mod constructors;

use minimum::systems::async_dispatch::{
    MinimumDispatcher,
    MinimumDispatcherBuilder
};

use minimum::component::Component;
use minimum::systems::World;

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

    //TODO-API: registering the component and then registering the component storage as a resource might be considered redundant

    //TODO-API: We could have a "default" entity set that doesn't need to be registered as a resource (and automatically knows about all component types)
    let mut entity_set = minimum::entity::EntitySet::new();
    entity_set.register_component_type::<components::PositionComponent>();
    entity_set.register_component_type::<components::VelocityComponent>();
    entity_set.register_component_type::<components::DebugDrawCircleComponent>();
    entity_set.register_component_type::<components::PlayerComponent>();
    entity_set.register_component_type::<components::BulletComponent>();
    entity_set.register_component_type::<components::FreeAtTimeComponent>();
    entity_set.register_component_type_with_free_handler::<components::PhysicsBodyComponent, components::PhysicsBodyComponentFreeHandler>();

    let mut world = minimum::systems::WorldBuilder::new()
        .with_resource(resources::GameControl::new())
        .with_resource(resources::DebugDraw::new())
        .with_resource(resources::InputManager::new())
        .with_resource(resources::TimeState::new())
        .with_resource(resources::PhysicsManager::new())
        .with_resource(window)
        .with_resource(resources::RenderState::empty())
        .with_resource(resources::DebugOptions::new())
        .with_resource(entity_set)
        .with_resource(<components::PositionComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::VelocityComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::DebugDrawCircleComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::PlayerComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::BulletComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::FreeAtTimeComponent as minimum::component::Component>::Storage::new())
        .with_resource(<components::PhysicsBodyComponent as minimum::component::Component>::Storage::new())
        .build();

    // Assets you want to always have available could be loaded here


    world.insert(init::init_imgui_manager(&world));
    world.insert(init::create_renderer(&world));

    create_objects(&world);

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

fn create_objects(world: &World) {
    let mut entities = world.fetch_mut::<minimum::EntitySet>();
    let mut pos_components = world.fetch_mut::<<components::PositionComponent as Component>::Storage>();
    let mut debug_draw_components = world.fetch_mut::<<components::DebugDrawCircleComponent as Component>::Storage>();
    let mut player_components = world.fetch_mut::<<components::PlayerComponent as Component>::Storage>();

    {
        let player_entity = entities.allocate_get();

        player_entity.add_component(
            &mut *player_components,
            components::PlayerComponent::new()
        );

        // Put a position on everything
        player_entity.add_component(
            &mut *pos_components,
            components::PositionComponent::new(glm::zero()),
        );

        player_entity.add_component(
            &mut *debug_draw_components,
            components::DebugDrawCircleComponent::new(15.0, glm::Vec4::new(0.0, 1.0, 0.0, 1.0)),
        );
    }

//    for i in 0..10 {
//        let entity = entities.allocate_get();
//
//        // Put a position on everything
//        entity.add_component(
//            &mut *pos_components,
//            components::PositionComponent::new(glm::Vec2::new(50.0 * i as f32, 6.0)),
//        );
//
//        entity.add_component(
//            &mut *vel_components,
//            components::DebugDrawCircleComponent::new(50.0, glm::Vec4::new(1.0, 0.0, 0.0, 1.0)),
//        );
//    }
}

fn dispatcher_thread(dispatcher: MinimumDispatcher) -> minimum::systems::World {
    info!("dispatch thread started");

    let mut world = dispatcher.enter_game_loop(move |dispatch_ctx| {

        let dispatch_context1 = dispatch_ctx.clone();
        let dispatch_context2 = dispatch_ctx.clone();

        //TODO: Explore non-intrusive method for defining task dependencies
        //TODO: Explore flags to turn steps on/off
        minimum::async_dispatcher::ExecuteSequential::new(vec![
            dispatch_ctx.run_task(tasks::ImguiBeginFrame),
            dispatch_ctx.run_task(tasks::UpdateTimeState),
            dispatch_ctx.run_task(tasks::GatherInput),
            dispatch_ctx.run_task(tasks::ControlPlayerEntity),
            dispatch_ctx.run_task(tasks::UpdatePositionWithVelocity),
            dispatch_ctx.run_task(tasks::HandleFreeAtTimeComponents),
            dispatch_ctx.run_task(tasks::UpdatePhysics),
            dispatch_ctx.run_task(tasks::UpdatePositionFromPhysics),
            dispatch_ctx.run_task(tasks::RenderImguiMainMenu),
            dispatch_ctx.run_task(tasks::UpdateDebugDraw),

            dispatch_ctx.visit_world(|world| {
                tasks::render(world);
            }),

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
