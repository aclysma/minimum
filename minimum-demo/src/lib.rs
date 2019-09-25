//There is a decent amount of dead code in this demo that is left as an example
#![allow(dead_code)]
extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

#[macro_use]
extern crate imgui_inspect_derive;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate strum_macros;

#[cfg(feature = "dim2")]
extern crate ncollide2d as ncollide;
#[cfg(feature = "dim3")]
extern crate ncollide3d as ncollide;
#[cfg(feature = "dim2")]
extern crate nphysics2d as nphysics;
#[cfg(feature = "dim3")]
extern crate nphysics3d as nphysics;

extern crate minimum_framework as framework;

mod components;
mod constructors;
#[cfg(feature = "editor")]
mod imgui_themes;
mod init;
mod renderer;
mod resources;
mod tasks;

use framework::resources::FrameworkActionQueue;
use framework::CloneComponentFactory;
use minimum::component::Component;
use minimum::{DispatchControl, UpdateLoopSingleThreaded, WorldBuilder};
use rendy::wsi::winit;

pub fn run_the_game() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("minimum::systems", log::LevelFilter::Warn)
        .filter_module("gfx_backend_metal", log::LevelFilter::Error)
        .filter_module("rendy", log::LevelFilter::Error)
        .init();

    // Any config/data you want to load before opening a window should go here

    let event_loop = winit::event_loop::EventLoop::<resources::WindowUserEvent>::with_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_title("Minimum Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1300.0, 900.0))
        .build(&event_loop)?;

    let mut world_builder = minimum::WorldBuilder::new()
        .with_resource(framework::resources::FrameworkActionQueue::new())
        .with_resource(framework::resources::DebugDraw::new())
        .with_resource(resources::InputManager::new())
        .with_resource(framework::resources::TimeState::new())
        .with_resource(resources::PhysicsManager::new())
        .with_resource(window)
        .with_resource(resources::RenderState::empty())
        .with_resource(framework::resources::FrameworkOptions::new())
        .with_component(<framework::components::TransformComponent as Component>::Storage::new())
        .with_component(<framework::components::VelocityComponent as Component>::Storage::new())
        .with_component(<framework::components::DebugDrawCircleComponent as Component>::Storage::new())
        .with_component(<framework::components::DebugDrawRectComponent as Component>::Storage::new())
        .with_component(<components::PlayerComponent as Component>::Storage::new())
        .with_component(<components::BulletComponent as Component>::Storage::new())
        .with_component(<framework::components::FreeAtTimeComponent as Component>::Storage::new())
        .with_component(
            <framework::components::PersistentEntityComponent as Component>::Storage::new(),
        )
        .with_component_and_free_handler::<_, _, components::PhysicsBodyComponentFreeHandler>(
            <components::PhysicsBodyComponent as Component>::Storage::new(),
        )
        //TODO: Ideally we don't need to register the factory in addition to the component itself
        .with_component_factory(CloneComponentFactory::<framework::components::TransformComponent>::new())
        .with_component_factory(CloneComponentFactory::<framework::components::VelocityComponent>::new())
        .with_component_factory(
            CloneComponentFactory::<framework::components::DebugDrawCircleComponent>::new(),
        )
        .with_component_factory(CloneComponentFactory::<framework::components::DebugDrawRectComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::PlayerComponent>::new())
        .with_component_factory(CloneComponentFactory::<components::BulletComponent>::new())
        .with_component_factory(CloneComponentFactory::<framework::components::FreeAtTimeComponent>::new())
        .with_component_factory(components::PhysicsBodyComponentFactory::new())
        .with_component_factory(CloneComponentFactory::<
            framework::components::PersistentEntityComponent,
        >::new());

    // Setup editor-only resources/components
    #[cfg(feature = "editor")]
    {
        world_builder = world_builder
                .with_component(<framework::components::editor::EditorModifiedComponent as Component>::Storage::new())
                .with_component(<framework::components::editor::EditorSelectedComponent as Component>::Storage::new())
                .with_resource(framework::resources::editor::EditorCollisionWorld::new())
                .with_resource(framework::resources::editor::EditorUiState::new())
                .with_resource(framework::resources::editor::EditorActionQueue::new())
                .with_resource(resources::EditorDraw::new())
                .with_component_and_free_handler::<_, _, framework::components::editor::EditorShapeComponentFreeHandler>(
                    <framework::components::editor::EditorShapeComponent as Component>::Storage::new(),
                )
                .with_component_factory(CloneComponentFactory::<framework::components::editor::EditorSelectedComponent>::new())
                .with_component_factory(framework::components::editor::EditorShapeComponentFactory::new());
    }

    // Register selectable types
    #[cfg(feature = "editor")]
    {
        let mut select_registry = framework::select::SelectRegistry::new();
        select_registry
            .register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>();
        select_registry
            .register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>();
        select_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawCircleComponent>>();
        select_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawRectComponent>>();

        world_builder.add_resource(select_registry);
    }

    // Register inspectable types
    #[cfg(feature = "editor")]
    {
        let mut inspect_registry = framework::inspect::InspectRegistry::new();
        inspect_registry.register_component::<framework::components::TransformComponent>("Position");
        inspect_registry.register_component::<framework::components::VelocityComponent>("Velocity");
        inspect_registry
            .register_component::<framework::components::DebugDrawCircleComponent>("Debug Draw Circle");
        inspect_registry
            .register_component::<framework::components::DebugDrawRectComponent>("Debug Draw Rectangle");
        inspect_registry.register_component::<components::BulletComponent>("Physics Body Box");
        inspect_registry
            .register_component::<components::PhysicsBodyComponent>("Physics Body Circle");
        inspect_registry.register_component::<components::PlayerComponent>("Player");

        inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::TransformComponent>>("Position");
        inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::VelocityComponent>>("Velocity");
        inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawCircleComponent>>("Debug Draw Circle");
        inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawRectComponent>>("Debug Draw Rectangle");
        inspect_registry
            .register_component_prototype::<components::PhysicsBodyComponentPrototypeCustom>(
                "Physics Body Custom",
            );
        inspect_registry
            .register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>(
                "Physics Body Box",
            );
        inspect_registry
            .register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>(
                "Physics Body Circle",
            );
        inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PlayerComponent>>("Player");

        world_builder.add_resource(inspect_registry);
    }

    // Register loadable/savable types
    let mut persist_registry = framework::persist::PersistRegistry::new();
    persist_registry.register_component_prototype::<framework::components::TransformComponentPrototype>("Position");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::VelocityComponent>>("Velocity");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawCircleComponent>>("Debug Draw Circle");
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<framework::components::DebugDrawRectComponent>>("Debug Draw Rectangle");
    persist_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>(
        "Physics Body Box",
    );
    persist_registry
        .register_component_prototype::<components::PhysicsBodyComponentPrototypeCircle>(
            "Physics Body Circle",
        );
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PlayerComponent>>("Player");
    world_builder.add_resource(persist_registry);

    register_tasks(&mut world_builder);

    let mut world = world_builder.build();

    #[cfg(feature = "editor")]
    {
        world
            .resource_map
            .insert(init::init_imgui_manager(&world.resource_map));
    }

    #[cfg(not(feature = "editor"))]
    {
        world.resource_map.insert(resources::ImguiManager {});
    }

    world
        .resource_map
        .insert(init::create_renderer(&world.resource_map));

    //create_objects(&resource_map);

    // Wrap the threadsafe interface to the window in WindowInterface and add it to the resource map
    // Return the event_tx which needs to be given to the event loop
    let winit_event_tx = init::create_window_interface(&mut world.resource_map, &event_loop);

    // Start the game loop thread
    let _join_handle = std::thread::spawn(|| dispatcher_thread(world));

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

fn register_tasks(world_builder: &mut WorldBuilder) {
    // Add the default phases
    world_builder.add_phase::<minimum::task::PhaseFrameBegin>();
    world_builder.add_phase::<minimum::task::PhaseGatherInput>();
    world_builder.add_phase::<minimum::task::PhasePrePhysicsGameplay>();
    world_builder.add_phase::<minimum::task::PhasePhysics>();
    world_builder.add_phase::<minimum::task::PhasePostPhysicsGameplay>();
    world_builder.add_phase::<minimum::task::PhasePreRender>();
    world_builder.add_phase::<minimum::task::PhaseRender>();
    world_builder.add_phase::<minimum::task::PhasePostRender>();
    world_builder.add_phase::<minimum::task::PhaseEndFrame>();

    // Add editor-only tasks
    #[cfg(feature = "editor")]
    {
        //This gets run at frame begin
        world_builder.add_task::<tasks::imgui::ImguiBeginFrameTask>();

        // This get run during pre-render
        world_builder.add_task::<tasks::imgui::RenderImguiMainMenuTask>();
        world_builder.add_task::<tasks::imgui::RenderImguiEntityListTask>();
        world_builder.add_task::<tasks::editor::EditorUpdateSelectionShapesWithPositionTask>();
        world_builder.add_task::<tasks::editor::EditorUpdateSelectionWorldTask>();
        world_builder.add_task::<tasks::editor::EditorHandleInputTask>();
        world_builder.add_task::<tasks::editor::EditorDrawSelectionShapesTask>();
        world_builder.add_task::<tasks::imgui::RenderImguiInspectorTask>();

        // This get run at end of frame
        world_builder.add_task::<tasks::editor::EditorUpdateActionQueueTask>();
        world_builder.add_task::<tasks::editor::EditorRecreateModifiedEntitiesTask>();
    }

    // Frame Begin
    world_builder.add_task::<framework::tasks::ClearDebugDrawTask>();
    world_builder.add_task::<tasks::UpdateTimeStateTask>();

    // Gather Input
    world_builder.add_task::<tasks::GatherInputTask>();

    // Pre Physics Gameplay
    world_builder.add_task::<tasks::ControlPlayerEntityTask>();
    world_builder.add_task::<framework::tasks::HandleFreeAtTimeComponentsTask>();
    world_builder.add_task::<tasks::UpdatePositionWithVelocityTask>();

    // Physics
    world_builder.add_task::<tasks::PhysicsSyncPreTask>();
    world_builder.add_task::<tasks::UpdatePhysicsTask>();
    world_builder.add_task::<tasks::PhysicsSyncPostTask>();

    // Pre-Render
    world_builder.add_task::<framework::tasks::DebugDrawComponentsTask>();

    // Render
    world_builder.add_task::<tasks::UpdateRendererTask>();

    // Frame End
    // This must be called once per frame to create/destroy entities
    world_builder.add_task::<framework::tasks::UpdateEntitySetTask>();
    world_builder.add_task::<framework::tasks::FrameworkUpdateActionQueueTask>();
}

fn dispatcher_thread(world: minimum::World) -> minimum::resource::ResourceMap {
    info!("dispatch thread started");

    // If editing, start paused
    #[cfg(feature = "editor")]
    let context_flags = framework::context_flags::AUTHORITY_CLIENT
        | framework::context_flags::AUTHORITY_SERVER
        | framework::context_flags::PLAYMODE_SYSTEM;

    // If not editing, start in playing mode
    #[cfg(not(feature = "editor"))]
    let context_flags = framework::context_flags::AUTHORITY_CLIENT
        | framework::context_flags::AUTHORITY_SERVER
        | framework::context_flags::PLAYMODE_PLAYING
        | framework::context_flags::PLAYMODE_PAUSED
        | framework::context_flags::PLAYMODE_SYSTEM;

    *world
        .resource_map
        .fetch_mut::<DispatchControl>()
        .next_frame_context_flags_mut() = context_flags;

    world
        .resource_map
        .fetch_mut::<FrameworkActionQueue>()
        .enqueue_load_level(std::path::PathBuf::from("test_save"));

    let update_loop = UpdateLoopSingleThreaded::new(world, context_flags);
    update_loop.run();

    let mut resource_map = update_loop.into_resource_map();

    // This would be a good spot to flush anything out like saved progress

    // Manual dispose is required for rendy
    let renderer = resource_map.remove::<renderer::Renderer>();
    renderer.unwrap().dispose(&resource_map);

    resource_map
        .fetch_mut::<resources::WindowInterface>()
        .event_loop_proxy
        .lock()
        .unwrap()
        .send_event(resources::WindowUserEvent::Terminate)
        .unwrap();

    info!("dispatch thread is done");
    resource_map
}
