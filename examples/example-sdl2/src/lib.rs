// There is "dead" example code in this crate
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use sdl2::event::Event;

use sdl2::mouse::MouseState;

use skulpin::{Sdl2Window, RendererBuilder};

use skulpin_plugin_imgui::ImguiRendererPlugin;

use atelier_assets::core::asset_uuid;
use atelier_assets::core as atelier_core;

use legion::*;

use minimum::resources::*;

mod systems;

mod registration;

use minimum::resources::editor::{
    EditorMode, EditorSelectionResource, EditorInspectRegistryResource, EditorStateResource,
    EditorDraw3DResource,
};
use minimum_sdl2::resources::{Sdl2WindowResource, Sdl2ImguiManagerResource};
use minimum_sdl2::imgui::Sdl2ImguiManager;
use minimum_skulpin::resources::CanvasDrawResource;
use example_shared::resources::FpsTextResource;
use minimum_nphysics2d::resources::PhysicsResource;
use atelier_assets::loader::rpc_loader::RpcLoader;

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
    let sdl2_imgui = minimum_sdl2::imgui::init_imgui_manager(&sdl2_window, minimum_sdl2::imgui::ColorFormat::Srgb);

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
    let mut world = World::default();
    let mut resources = create_resources(&sdl2_window, &sdl2_imgui);

    // Start the application
    EditorStateResource::open_prefab(
        &mut world,
        &resources,
        asset_uuid!("3991506e-ed7e-4bcb-8cfd-3366b31a6439"),
    )
    .unwrap();

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
                minimum_sdl2::input::handle_sdl2_event(
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

        renderer
            .draw(&skulpin_window, |canvas, coordinate_system_helper| {
                resources
                    .get_mut::<CanvasDrawResource>()
                    .unwrap()
                    .begin_draw_context(canvas, coordinate_system_helper);

                draw_schedule.execute(&mut world, &mut resources);

                resources
                    .get_mut::<CanvasDrawResource>()
                    .unwrap()
                    .end_draw_context();
            })
            .unwrap();

        if resources
            .get::<AppControlResource>()
            .unwrap()
            .should_terminate_process()
        {
            break;
        }
    }

    // Drop world before resources as physics components may point back at resources
    std::mem::drop(world);
    std::mem::drop(resources);
}

fn create_resources(
    sdl2_window: &sdl2::video::Window,
    sdl2_imgui: &Sdl2ImguiManager,
) -> Resources {
    let mut resources = Resources::default();

    let rpc_loader = RpcLoader::new("127.0.0.1:9999".to_string()).unwrap();
    let asset_resource = registration::create_asset_manager(rpc_loader);

    let physics_resource = PhysicsResource::new(glam::Vec2::unit_y() * GRAVITY);

    let camera_resource = CameraResource::new(
        glam::Vec2::new(0.0, 1.0),
        crate::GROUND_HALF_EXTENTS_WIDTH * 1.5,
    );

    let sdl2_window_resource = Sdl2WindowResource::new(sdl2_window);

    let drawable = sdl2_window_resource.drawable_size();

    let mut viewport = ViewportResource::empty();
    let viewport_size_in_pixels = glam::Vec2::new(drawable.width as f32, drawable.height as f32);
    example_shared::viewport::update_viewport(
        &mut viewport,
        viewport_size_in_pixels,
        camera_resource.position,
        camera_resource.x_half_extents
    );

    resources.insert(viewport);
    resources.insert(DebugDraw2DResource::new());
    resources.insert(DebugDraw3DResource::new());
    resources.insert(EditorDraw3DResource::new());
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
    resources.insert(Sdl2WindowResource::new(&sdl2_window));
    resources.insert(EditorStateResource::new());

    use minimum_sdl2::input::Sdl2KeyboardKey;
    use sdl2::keyboard::Keycode;
    let keybinds = minimum::resources::editor::Keybinds {
        selection_add: Sdl2KeyboardKey::new(Keycode::LShift).into(),
        selection_subtract: Sdl2KeyboardKey::new(Keycode::LAlt).into(),
        selection_toggle: Sdl2KeyboardKey::new(Keycode::LCtrl).into(),
        tool_translate: Sdl2KeyboardKey::new(Keycode::Num1).into(),
        tool_scale: Sdl2KeyboardKey::new(Keycode::Num2).into(),
        tool_rotate: Sdl2KeyboardKey::new(Keycode::Num3).into(),
        action_quit: Sdl2KeyboardKey::new(Keycode::Escape).into(),
        action_toggle_editor_pause: Sdl2KeyboardKey::new(Keycode::Space).into(),
    };

    resources.insert(minimum::resources::editor::EditorSettingsResource::new(
        keybinds,
    ));
    resources
}
