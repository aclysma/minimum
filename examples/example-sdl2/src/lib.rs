#[macro_use]
extern crate log;

#[macro_use]
extern crate itertools;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseState;

use skulpin::{Sdl2Window, RendererBuilder, CoordinateSystemHelper};
use skulpin::skia_safe;
use skulpin_plugin_imgui::ImguiRendererPlugin;

use atelier_assets::core::asset_uuid;
use atelier_assets::core as atelier_core;

use legion::prelude::*;

use minimum::resources::*;

mod components;

mod resources;
use resources::*;

mod systems;

mod registration;

mod sdl2_imgui;
use sdl2_imgui::Sdl2ImguiManager;
use minimum::resources::editor::{
    EditorMode, EditorSelectionResource, EditorInspectRegistryResource, EditorStateResource,
    EditorDrawResource,
};

mod sdl2_input;

pub const GRAVITY: f32 = -9.81;
pub const GROUND_HALF_EXTENTS_WIDTH: f32 = 3.0;

pub fn run() {
    // Init SDL2
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    // Create a window
    let window_size = (900, 600);
    let sdl2_window = video_subsystem
        .window("Minimum SDL2 Example", window_size.0, window_size.1)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .vulkan()
        .build()
        .expect("Failed to create window");

    // Init imgui
    let sdl2_imgui = sdl2_imgui::init_imgui_manager(&sdl2_window);

    // Setup skulpin imgui plugin
    let mut imgui_plugin: Option<Box<dyn skulpin::RendererPlugin>> = None;
    sdl2_imgui.with_context(|context| {
        imgui_plugin = Some(Box::new(ImguiRendererPlugin::new(context)));
    });

    // Configuration for skulpin/skia canvas
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: window_size.0 as f32,
        top: 0.0,
        bottom: window_size.1 as f32,
    };

    // Set up the skulpin renderer
    let skulpin_window = Sdl2Window::new(&sdl2_window);
    let renderer = RendererBuilder::new()
        .use_vulkan_debug_layer(true)
        .add_plugin(imgui_plugin.unwrap())
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&skulpin_window);

    // Check if there were error setting up the renderer
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    log::info!("renderer created");
    let mut renderer = renderer.unwrap();

    // Create the event pump
    log::info!("Starting window event loop");
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not create sdl event pump");

    // Create a legion world
    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = create_resources(universe, &sdl2_window, &sdl2_imgui);

    // Start the application
    EditorStateResource::open_prefab(
        &mut world,
        &resources,
        asset_uuid!("3991506e-ed7e-4bcb-8cfd-3366b31a6439"),
    );

    let schedule_criteria = systems::ScheduleCriteria::new(false, EditorMode::Active);
    let mut update_schedule = systems::create_update_schedule(&schedule_criteria);
    let mut draw_schedule = systems::create_draw_schedule(&schedule_criteria);

    // Run the event loop
    'running: loop {
        for event in event_pump.poll_iter() {
            log::info!("SDL2 Event: {:?}", event);

            sdl2_imgui.handle_event(&event);

            if !sdl2_imgui.ignore_event(&event) {
                let mut input_resource = resources.get_mut::<InputResource>().unwrap();
                let viewport = resources.get_mut::<ViewportResource>().unwrap();
                crate::sdl2_input::handle_sdl2_event(
                    &event,
                    input_resource.input_state_mut(),
                    &*viewport,
                );

                match event {
                    //
                    // Halt if the user requests to close the window
                    //
                    Event::Quit { .. } => break 'running,

                    _ => {}
                }
            }
        }

        // Update/Draw here
        sdl2_imgui.begin_frame(&sdl2_window, &MouseState::new(&event_pump));

        update_schedule.execute(&mut world, &mut resources);

        sdl2_imgui.render(&sdl2_window);

        renderer.draw(&skulpin_window, |canvas, coordinate_system_helper| {
            resources
                .get_mut::<CanvasDrawResource>()
                .unwrap()
                .begin_draw_context(canvas, coordinate_system_helper);

            draw_schedule.execute(&mut world, &mut resources);

            resources
                .get_mut::<CanvasDrawResource>()
                .unwrap()
                .end_draw_context();
        });

        if resources
            .get::<AppControlResource>()
            .unwrap()
            .should_terminate_process()
        {
            break;
        }
    }
}

fn create_resources(
    universe: Universe,
    sdl2_window: &sdl2::video::Window,
    sdl2_imgui: &Sdl2ImguiManager,
) -> Resources {
    let mut resources = Resources::default();

    let asset_resource = registration::create_asset_manager();

    let physics_resource = PhysicsResource::new(glam::Vec2::unit_y() * GRAVITY);

    let mut camera_resource = CameraResource::new(
        glam::Vec2::new(0.0, 1.0),
        crate::GROUND_HALF_EXTENTS_WIDTH * 1.5,
    );

    let sdl2_window_resource = Sdl2WindowResource::new(sdl2_window);

    let drawable = sdl2_window_resource.drawable_size();
    let viewport_size = ViewportSize::new(drawable.width, drawable.height);
    resources.insert(ViewportResource::new(
        viewport_size,
        camera_resource.position,
        camera_resource.x_half_extents,
    ));
    resources.insert(DebugDrawResource::new());
    resources.insert(EditorDrawResource::new());
    resources.insert(EditorSelectionResource::new(
        registration::create_editor_selection_registry(),
    ));
    resources.insert(EditorInspectRegistryResource::new(
        registration::create_editor_inspector_registry(),
    ));
    resources.insert(ComponentRegistryResource::new(
        registration::create_component_registry(),
    ));
    resources.insert(FpsTextResource::new());
    resources.insert(asset_resource);
    resources.insert(physics_resource);
    resources.insert(camera_resource);
    resources.insert(Sdl2ImguiManagerResource::new(sdl2_imgui.clone()));
    resources.insert(ImguiResource::new(sdl2_imgui.imgui_manager()));
    resources.insert(AppControlResource::new());
    resources.insert(TimeResource::new());
    resources.insert(InputResource::new());
    resources.insert(CanvasDrawResource::default());
    resources.insert(UniverseResource::new(universe));
    resources.insert(Sdl2WindowResource::new(&sdl2_window));
    resources.insert(EditorStateResource::new());
    resources
}
