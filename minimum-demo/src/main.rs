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

use minimum::dispatch::async_dispatch::MinimumDispatcher;

use framework::CloneComponentFactory;
//use framework::CloneComponentPrototypeSerializer;
use minimum::component::Component;
use minimum::resource::ResourceMap;

#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount)]
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
        .build(&event_loop)?;

    let mut resource_map = minimum::WorldBuilder::new()
        .with_resource(resources::GameControl::new())
        .with_resource(resources::DebugDraw::new())
        .with_resource(resources::InputManager::new())
        .with_resource(resources::TimeState::new())
        .with_resource(resources::PhysicsManager::new())
        .with_resource(window)
        .with_resource(resources::RenderState::empty())
        .with_resource(resources::DebugOptions::new())
        .with_resource(resources::EditorCollisionWorld::new())
        .with_resource(resources::EditorUiState::new())
        .with_component(<components::PositionComponent as Component>::Storage::new())
        .with_component(<components::VelocityComponent as Component>::Storage::new())
        .with_component(<components::DebugDrawCircleComponent as Component>::Storage::new())
        .with_component(<components::DebugDrawRectComponent as Component>::Storage::new())
        .with_component(<components::PlayerComponent as Component>::Storage::new())
        .with_component(<components::BulletComponent as Component>::Storage::new())
        .with_component(<components::FreeAtTimeComponent as Component>::Storage::new())
        .with_component(<components::EditorSelectedComponent as Component>::Storage::new())
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
    inspect_registry.register_component::<components::PositionComponent>();
    inspect_registry.register_component::<components::VelocityComponent>();
    inspect_registry.register_component::<components::DebugDrawCircleComponent>();
    inspect_registry.register_component::<components::DebugDrawRectComponent>();
    inspect_registry.register_component::<components::BulletComponent>();

    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PositionComponent>>();
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::VelocityComponent>>();
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawCircleComponent>>();
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawRectComponent>>();
    inspect_registry.register_component_prototype::<framework::CloneComponentPrototype<components::BulletComponent>>();
    inspect_registry
        .register_component_prototype::<components::PhysicsBodyComponentPrototypeCustom>();
    inspect_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>();

    let mut persist_registry = framework::persist::PersistRegistry::new();

    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::PositionComponent>>();
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::VelocityComponent>>();
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawCircleComponent>>();
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::DebugDrawRectComponent>>();
    persist_registry.register_component_prototype::<framework::CloneComponentPrototype<components::BulletComponent>>();
    persist_registry.register_component_prototype::<components::PhysicsBodyComponentPrototypeBox>();
    //persist_registry.register_component_prototype::<components::PhysicsBodyComponentPrototype>();
    //persist_registry.register_component_prototype::<components::EditorShapeComponentPrototype>();

    // Assets you want to always have available could be loaded here

    resource_map.insert(inspect_registry);
    resource_map.insert(persist_registry);
    resource_map.insert(init::init_imgui_manager(&resource_map));
    resource_map.insert(init::create_renderer(&resource_map));

    create_objects(&resource_map);

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

fn create_objects(resource_map: &ResourceMap) {
    let mut entity_factory = resource_map.fetch_mut::<minimum::EntityFactory>();
    constructors::create_player(&mut *entity_factory);

    constructors::create_wall(
        glm::vec2(250.0, -125.0),
        glm::vec2(20.0, 100.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(200.0, 250.0),
        glm::vec2(30.0, 50.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(-170.0, 40.0),
        glm::vec2(50.0, 100.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(-120.0, -100.0),
        glm::vec2(100.0, 10.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(10.0, -200.0),
        glm::vec2(15.0, 40.0),
        &mut *entity_factory,
    );

    constructors::create_wall(
        glm::vec2(0.0, 280.0),
        glm::vec2(800.0, 10.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(0.0, -280.0),
        glm::vec2(800.0, 10.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(380.0, 0.0),
        glm::vec2(10.0, 600.0),
        &mut *entity_factory,
    );
    constructors::create_wall(
        glm::vec2(-380.0, 0.0),
        glm::vec2(10.0, 600.0),
        &mut *entity_factory,
    );
}

fn dispatcher_thread(
    resource_map: minimum::resource::ResourceMap,
) -> minimum::resource::ResourceMap {
    info!("dispatch thread started");

    let context_flags = crate::context_flags::AUTHORITY_CLIENT
        | crate::context_flags::AUTHORITY_SERVER
        | crate::context_flags::PLAYMODE_PLAYING
        | crate::context_flags::PLAYMODE_PAUSED
        | crate::context_flags::PLAYMODE_SYSTEM;

    let dispatcher = MinimumDispatcher::new(resource_map, context_flags);
    let mut resource_map = dispatcher.enter_game_loop(move |dispatch_ctx| {
        //TODO: Explore non-intrusive method for defining task dependencies
        //TODO: Explore flags to turn steps on/off
        minimum::async_dispatch::ExecuteSequential::new(vec![
            dispatch_ctx.run_task(tasks::ClearDebugDraw),
            dispatch_ctx.run_task(tasks::ImguiBeginFrame),
            dispatch_ctx.run_task(tasks::UpdateTimeState),
            dispatch_ctx.run_task(tasks::GatherInput),
            dispatch_ctx.run_task(tasks::ControlPlayerEntity),
            dispatch_ctx.run_task(tasks::UpdatePositionWithVelocity),
            dispatch_ctx.run_task(tasks::HandleFreeAtTimeComponents),
            dispatch_ctx.run_task(tasks::PhysicsSyncPre),
            dispatch_ctx.run_task(tasks::UpdatePhysics),
            dispatch_ctx.run_task(tasks::PhysicsSyncPost),
            dispatch_ctx.run_task(tasks::RenderImguiMainMenu),
            dispatch_ctx.run_task(tasks::RenderImguiEntityList),
            dispatch_ctx.run_task(tasks::EditorUpdateShapesWithPosition),
            dispatch_ctx.run_task(tasks::EditorUpdateCollisionWorld),
            dispatch_ctx.run_task(tasks::EditorHandleInput),
            dispatch_ctx.run_task(tasks::EditorDrawShapes),
            dispatch_ctx.run_task(tasks::DebugDrawComponents),
            dispatch_ctx.visit_resources(|resource_map| {
                // Draw Inspector
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("inspect");
                    draw_inspector(&resource_map);
                }

                // Render
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("render");
                    render(resource_map);
                }

                // This must be called once per frame to create/destroy entities
                {
                    let _scope_timer = minimum::util::ScopeTimer::new("entity update");
                    update_entity_set(resource_map);
                }
            }),
            // This checks if we need to load a different level or kill the process
            dispatch_ctx.visit_resources_mut(move |resource_map| {
                let _scope_timer = minimum::util::ScopeTimer::new("end frame");
                end_frame(resource_map)
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

pub fn render(resource_map: &ResourceMap) {
    let window = resource_map.fetch::<winit::window::Window>();
    let mut renderer = resource_map.fetch_mut::<crate::renderer::Renderer>();
    renderer.render(&window, &resource_map);
}

fn draw_inspector(resource_map: &ResourceMap) {
    let play_mode = resource_map.fetch::<resources::TimeState>().play_mode;
    let mut editor_ui_state = resource_map.fetch_mut::<resources::EditorUiState>();
    let window_options = editor_ui_state.window_options(play_mode);
    if !window_options.show_inspector {
        return;
    }

    let entity_set = resource_map.fetch::<minimum::EntitySet>();
    let selected_entity_handles = {
        let selected_components =
            resource_map.fetch_mut::<<components::EditorSelectedComponent as Component>::Storage>();
        let mut selected = vec![];
        for (entity_handle, _) in selected_components.iter(&entity_set) {
            selected.push(entity_handle);
        }
        selected
    };

    let inspect_registry = resource_map.fetch::<framework::inspect::InspectRegistry>();
    let mut imgui_manager = resource_map.fetch_mut::<resources::ImguiManager>();
    imgui_manager.with_ui(|ui| {

        use imgui::im_str;


        //ui.set
        ui.window(im_str!("Inspector"))
            .position([0.0, 250.0], imgui::Condition::Once)
            .size([200.0, 300.0], imgui::Condition::Once)
            .build(|| {

                if ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]) {
                    ui.open_popup(im_str!("Add Component"));
                }

                ui.popup(im_str!("Add Component"), || {
                    //ui.text("Content of my modal");

                    ui.text("abc");
                    ui.input_text(im_str!("Search"), &mut editor_ui_state.add_component_search_text)
                        //.auto_select_all(true)
                        //.resize_buffer(true)
                        .build();
                    ui.text("xyz");
                    if ui.button(im_str!("OK"), [0.0, 0.0]) {
                        ui.close_current_popup();
                    }
                });


                inspect_registry.render_mut(resource_map, selected_entity_handles.as_slice(), ui);
            })
    });
}

fn update_entity_set(resource_map: &ResourceMap) {
    let mut entity_set = resource_map.fetch_mut::<minimum::entity::EntitySet>();
    entity_set.update(resource_map);
}

fn end_frame(resource_map: &mut ResourceMap) {
    let mut game_control = resource_map.fetch_mut::<resources::GameControl>();
    let mut dispatch_control = resource_map.fetch_mut::<minimum::DispatchControl>();

    if game_control.terminate_process() {
        dispatch_control.end_game_loop();
    } else if game_control.has_load_level() {
        // Unload game state
        let mut entity_set = resource_map.fetch_mut::<minimum::entity::EntitySet>();
        entity_set.clear(resource_map);
        //resource_map.remove::<game::GameState>();

        // Setup game state
        let _level_to_load = game_control.take_load_level();
        //resource_map.insert::<physics::Physics>();
    }

    if let Some(_save_path) = game_control.take_save_level() {
        println!("save");

        //let entity_set = resource_map.fetch::<minimum::EntitySet>();

        let persist_registry = resource_map.fetch::<framework::persist::PersistRegistry>();
        persist_registry.save(resource_map);
    }
}
