//There is a decent amount of dead code in this demo that is left as an example
#![allow(dead_code)]

extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

#[macro_use]
extern crate named_type_derive;

#[macro_use]
extern crate imgui_inspect_derive;

#[macro_use]
extern crate mopa;

//#[macro_use]
//extern crate minimum_derive;

mod components;
mod constructors;
mod framework;
mod imgui_themes;
mod init;
mod renderer;
mod resources;
mod tasks;
mod update;

use minimum::dispatch::async_dispatch::MinimumDispatcher;

use framework::CloneComponentFactory;
use minimum::component::Component;
use resources::EditorActionQueue;

#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount, Debug)]
pub enum PlayMode {
    // Represents the game being frozen for debug purposes
    System,

    // Represents the game being puased by the user (actual meaning of this is game-specific)
    Paused,

    // Normal simulation is running
    Playing,
}

//PLAYMODE_COUNT exists due to strum_macros::EnumCount
const PLAY_MODE_COUNT: usize = PLAYMODE_COUNT;

pub mod context_flags {
    // For pause status. Flags will be set based on if the game is in a certain playmode
    pub const PLAYMODE_SYSTEM: usize = 1;
    pub const PLAYMODE_PAUSED: usize = 2;
    pub const PLAYMODE_PLAYING: usize = 4;

    // For multiplayer games:
    // - Dedicated Server will only run Net_Server
    // - Pure client will only have Net_Client
    // - "Listen" client will have both
    // - Singleplayer will have both
    pub const AUTHORITY_SERVER: usize = 8;
    pub const AUTHORITY_CLIENT: usize = 16;
}

fn main() {
    run_the_game().unwrap();
}

fn run_the_game() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("minimum::systems", log::LevelFilter::Warn)
        .filter_module("gfx_backend_metal", log::LevelFilter::Error)
        .filter_module("rendy", log::LevelFilter::Error)
        .init();

    // Any config/data you want to load before opening a window should go here

    let event_loop = winit::event_loop::EventLoop::<resources::WindowUserEvent>::new_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_title("Minimum Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1300.0, 900.0))
        .build(&event_loop)?;

    let mut resource_map = minimum::WorldBuilder::new()
        .with_resource(resources::FrameworkActionQueue::new())
        .with_resource(resources::DebugDraw::new())
        .with_resource(resources::InputManager::new())
        .with_resource(resources::TimeState::new())
        .with_resource(resources::PhysicsManager::new())
        .with_resource(window)
        .with_resource(resources::RenderState::empty())
        .with_resource(resources::DebugOptions::new())
        .with_resource(resources::EditorCollisionWorld::new())
        .with_resource(resources::EditorUiState::new())
        .with_resource(resources::EditorActionQueue::new())
        .with_component(<components::PositionComponent as Component>::Storage::new())
        .with_component(<components::VelocityComponent as Component>::Storage::new())
        .with_component(<components::DebugDrawCircleComponent as Component>::Storage::new())
        .with_component(<components::DebugDrawRectComponent as Component>::Storage::new())
        .with_component(<components::PlayerComponent as Component>::Storage::new())
        .with_component(<components::BulletComponent as Component>::Storage::new())
        .with_component(<components::FreeAtTimeComponent as Component>::Storage::new())
        .with_component(<components::EditorSelectedComponent as Component>::Storage::new())
        .with_component(<components::EditorModifiedComponent as Component>::Storage::new())
        .with_component(<components::PersistentEntityComponent as Component>::Storage::new())
        .with_component_and_free_handler::<_, _, components::PhysicsBodyComponentFreeHandler>(
            <components::PhysicsBodyComponent as Component>::Storage::new(),
        )
        .with_component_and_free_handler::<_, _, components::EditorShapeComponentFreeHandler>(
            <components::EditorShapeComponent as Component>::Storage::new(),
        )
        //TODO: Ideally we don't need to register the factory in addition to the component itself
        .with_component_factory(CloneComponentFactory::<components::PositionComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::VelocityComponent>::new())
        .with_component_factory(
            CloneComponentFactory::<components::DebugDrawCircleComponent>::new(),
        )
        .with_component_factory(CloneComponentFactory::<components::DebugDrawRectComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::PlayerComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::BulletComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::FreeAtTimeComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::EditorSelectedComponent>::new())
        .with_component_factory(components::PhysicsBodyComponentFactory::new())
        .with_component_factory(components::EditorShapeComponentFactory::new())
        .with_component_factory(
            CloneComponentFactory::<components::PersistentEntityComponent>::new(),
        )
        .build();

    let mut inspect_registry = framework::inspect::InspectRegistry::new();
    inspect_registry.register_component::<components::PositionComponent>("Position");
    inspect_registry.register_component::<components::VelocityComponent>("Velocity");
    inspect_registry.register_component::<components::DebugDrawCircleComponent>("Debug Draw Circle");
    inspect_registry.register_component::<components::DebugDrawRectComponent>("Debug Draw Rectangle");
    inspect_registry.register_component::<components::BulletComponent>("Physics Body Box");
    inspect_registry.register_component::<components::PhysicsBodyComponent>("Physics Body Circle");
    inspect_registry.register_component::<components::PlayerComponent>("Player");

    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PositionComponent>>("Position");
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::VelocityComponent>>("Velocity");
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawCircleComponent>>("Debug Draw Circle");
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawRectComponent>>("Debug Draw Rectangle");
    inspect_registry
        .register_component_prototype::<components::PhysicsBodyComponentPrototypeCustom>("Physics Body Custom");
    inspect_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>("Physics Body Box");
    inspect_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>("Physics Body Circle");
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PlayerComponent>>("Player");

    let mut persist_registry = framework::persist::PersistRegistry::new();
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PositionComponent>>("Position");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::VelocityComponent>>("Velocity");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawCircleComponent>>("Debug Draw Circle");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawRectComponent>>("Debug Draw Rectangle");
    persist_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>("Physics Body Box");
    persist_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>("Physics Body Circle");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PlayerComponent>>("Player");

    let mut select_registry = framework::select::SelectRegistry::new();
    select_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>();
    select_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>();
    select_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawCircleComponent>>();
    select_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawRectComponent>>();

    resource_map.insert(inspect_registry);
    resource_map.insert(persist_registry);
    resource_map.insert(select_registry);
    resource_map.insert(init::init_imgui_manager(&resource_map));
    resource_map.insert(init::create_renderer(&resource_map));

    //create_objects(&resource_map);

    // Wrap the threadsafe interface to the window in WindowInterface and add it to the resource map
    // Return the event_tx which needs to be given to the event loop
    let winit_event_tx = init::create_window_interface(&mut resource_map, &event_loop);

    // Start the game loop thread
    let _join_handle = std::thread::spawn(|| dispatcher_thread(resource_map));

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

fn dispatcher_thread(
    resource_map: minimum::resource::ResourceMap,
) -> minimum::resource::ResourceMap {
    info!("dispatch thread started");

    // Start off in the editor state
    let context_flags = crate::context_flags::AUTHORITY_CLIENT
        | crate::context_flags::AUTHORITY_SERVER
        //| crate::context_flags::PLAYMODE_PLAYING
        //| crate::context_flags::PLAYMODE_PAUSED
        | crate::context_flags::PLAYMODE_SYSTEM;

    let dispatcher = MinimumDispatcher::new(resource_map, context_flags);
    let mut resource_map = dispatcher.enter_game_loop(move |dispatch_ctx| {
        //TODO: Explore non-intrusive method for defining task dependencies
        //TODO: Explore flags to turn steps on/off
        minimum::async_dispatch::ExecuteSequential::new(vec![
            // Pre Input
            dispatch_ctx.run_task(tasks::ClearDebugDraw),
            dispatch_ctx.run_task(tasks::ImguiBeginFrame),
            dispatch_ctx.run_task(tasks::UpdateTimeState),

            // Input
            dispatch_ctx.run_task(tasks::GatherInput),

            // Pre Physics
            dispatch_ctx.run_task(tasks::ControlPlayerEntity),
            dispatch_ctx.run_task(tasks::HandleFreeAtTimeComponents),
            dispatch_ctx.run_task(tasks::UpdatePositionWithVelocity),

            // Physics
            dispatch_ctx.run_task(tasks::PhysicsSyncPre),
            dispatch_ctx.run_task(tasks::UpdatePhysics),
            dispatch_ctx.run_task(tasks::PhysicsSyncPost),

            // Post Physics

            // Pre Render
            dispatch_ctx.run_task(tasks::RenderImguiMainMenu),
            dispatch_ctx.run_task(tasks::RenderImguiEntityList),
            dispatch_ctx.run_task(tasks::EditorUpdateSelectionShapesWithPosition),
            dispatch_ctx.run_task(tasks::EditorUpdateSelectionWorld),
            dispatch_ctx.run_task(tasks::EditorHandleInput),
            dispatch_ctx.run_task(tasks::EditorDrawSelectionShapes),
            dispatch_ctx.run_task(tasks::DebugDrawComponents),
            dispatch_ctx.visit_resources(|resource_map| {
                // Draw Inspector
                {
                    // This requires global data access since we're going to draw/edit potentially any
                    // component
                    let _scope_timer = minimum::util::ScopeTimer::new("inspect");
                    framework::inspect::draw_inspector(&resource_map);
                }

                // Render
                {
                    // This could potentially take a subset of the data, but it's more convenient
                    // to pass everything
                    let _scope_timer = minimum::util::ScopeTimer::new("render");
                    update::render(resource_map);
                }

                // This must be called once per frame to create/destroy entities
                {
                    // Updating the entity set will process queued work like creating/deleting components
                    // This could require access to any component type
                    let _scope_timer = minimum::util::ScopeTimer::new("entity update");
                    update::update_entity_set(resource_map);
                }
            }),
            // This checks if we need to load a different level or kill the process
            dispatch_ctx.visit_resources_mut(move |resource_map| {

                // Drain the editor action queue. This can potentially add/remove entities
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("editor queue");
                    let mut editor_action_queue = resource_map.fetch_mut::<EditorActionQueue>();
                    editor_action_queue.process_queue(resource_map);
                }

                // Drain the framework queue. This can potentiall load/save/reset the game state
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("framework_action_queue");
                    let mut framework_action_queue = resource_map.fetch_mut::<resources::FrameworkActionQueue>();
                    framework_action_queue.process_queue(resource_map);
                }

                // Rebuild any entities that had their prototype changed
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("recreate_modified_entities");
                    update::recreate_editor_modified_entities(resource_map);
                }
            }),
        ])
    });

    // This would be a good spot to flush anything out like saved progress

    // Manual dispose is required for rendy
    let renderer = resource_map.remove::<renderer::Renderer>();
    renderer.unwrap().dispose(&resource_map);

    resource_map
        .fetch_mut::<resources::WindowInterface>()
        .event_loop_proxy
        .send_event(resources::WindowUserEvent::Terminate)
        .unwrap();

    info!("dispatch thread is done");
    resource_map
}

