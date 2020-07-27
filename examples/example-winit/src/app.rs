use skulpin::{RendererBuilder, Window};

use skulpin::CreateRendererError;
use skulpin::ash;
use skulpin::winit;

use minimum_winit::imgui::WinitImguiManager;

use legion::prelude::*;
use skulpin_plugin_imgui::ImguiRendererPlugin;
use minimum::resources::{
    ImguiResource, AppControlResource, TimeResource, InputResource, UniverseResource,
    ViewportResource,
};
use minimum_winit::resources::{WinitImguiManagerResource, WinitWindowResource};
use minimum_skulpin::resources::CanvasDrawResource;

/// Represents an error from creating the renderer
#[derive(Debug)]
pub enum AppError {
    CreateRendererError(CreateRendererError),
    VkError(ash::vk::Result),
    WinitError(winit::error::OsError),
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            AppError::CreateRendererError(ref e) => Some(e),
            AppError::VkError(ref e) => Some(e),
            AppError::WinitError(ref e) => Some(e),
        }
    }
}

impl core::fmt::Display for AppError {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        match *self {
            AppError::CreateRendererError(ref e) => e.fmt(fmt),
            AppError::VkError(ref e) => e.fmt(fmt),
            AppError::WinitError(ref e) => e.fmt(fmt),
        }
    }
}

impl From<CreateRendererError> for AppError {
    fn from(result: CreateRendererError) -> Self {
        AppError::CreateRendererError(result)
    }
}

impl From<ash::vk::Result> for AppError {
    fn from(result: ash::vk::Result) -> Self {
        AppError::VkError(result)
    }
}

impl From<winit::error::OsError> for AppError {
    fn from(result: winit::error::OsError) -> Self {
        AppError::WinitError(result)
    }
}

pub trait AppHandler {
    /// Called once at start, put one-time init code here
    fn init(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        window: &dyn Window,
    );

    /// Called frequently, this is the intended place to put non-rendering logic
    fn update(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    );

    /// Called frequently, this is the intended place to put drawing code
    fn draw(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    );

    fn fatal_error(
        &mut self,
        error: &AppError,
    );
}

pub struct App {}

impl App {
    /// Runs the app. This is called by `AppBuilder::run`. This does not return because winit does
    /// not return. For consistency, we use the fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + AppHandler, S: Into<winit::dpi::Size>>(
        mut app_handler: T,
        logical_size: S,
        renderer_builder: RendererBuilder,
    ) -> ! {
        // Create the event loop
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

        // Create a single window
        let window_result = winit::window::WindowBuilder::new()
            .with_title("Minimum Winit Example")
            .with_inner_size(logical_size)
            .build(&event_loop);

        let winit_window = match window_result {
            Ok(window) => window,
            Err(e) => {
                log::warn!("Passing WindowBuilder::build() error to app {}", e);

                let app_error = e.into();
                app_handler.fatal_error(&app_error);

                // Exiting in this way is consistent with how we will exit if we fail within the
                // input loop
                std::process::exit(0);
            }
        };

        // Initialize imgui
        let winit_imgui_manager = init_winit_imgui_manager(&winit_window);

        // Initialize an interface for skulpin to interact with imgui
        let mut imgui_plugin: Option<Box<dyn skulpin::RendererPlugin>> = None;
        winit_imgui_manager.with_context(|context| {
            imgui_plugin = Some(Box::new(ImguiRendererPlugin::new(context)));
        });

        let skulpin_window = skulpin::WinitWindow::new(&winit_window);

        let renderer_result = renderer_builder
            .add_plugin(imgui_plugin.unwrap())
            .build(&skulpin_window);

        let mut renderer = match renderer_result {
            Ok(renderer) => renderer,
            Err(e) => {
                log::warn!("Passing RendererBuilder::build() error to app {}", e);

                let app_error = e.into();
                app_handler.fatal_error(&app_error);

                // Exiting in this way is consistent with how we will exit if we fail within the
                // input loop
                std::process::exit(0);
            }
        };

        let universe = Universe::new();
        let mut world = universe.create_world();
        let mut resources = legion::systems::resource::Resources::default();

        resources.insert(WinitImguiManagerResource::new(winit_imgui_manager.clone()));
        resources.insert(ImguiResource::new(winit_imgui_manager.imgui_manager()));
        resources.insert(AppControlResource::new());
        resources.insert(TimeResource::new());
        resources.insert(InputResource::new());
        resources.insert(CanvasDrawResource::default());
        resources.insert(UniverseResource::new(universe));
        resources.insert(WinitWindowResource::new(&winit_window));

        app_handler.init(&mut world, &mut resources, &skulpin_window);

        // Pass control of this thread to winit until the app terminates. If this app wants to quit,
        // the update loop should send the appropriate event via the channel
        event_loop.run(move |event, _window_target, control_flow| {
            // Let imgui have the event first
            let input_captured = {
                let winit_imgui_manager = resources.get_mut::<WinitImguiManagerResource>().unwrap();
                winit_imgui_manager.handle_event(&winit_window, &event);

                let mut input_captured = false;
                input_captured |= winit_imgui_manager.want_capture_keyboard()
                    && match event {
                        winit::event::Event::WindowEvent {
                            event: winit::event::WindowEvent::KeyboardInput { .. },
                            ..
                        } => true,
                        _ => false,
                    };

                input_captured |= winit_imgui_manager.want_capture_mouse()
                    && match event {
                        winit::event::Event::WindowEvent {
                            event: winit::event::WindowEvent::MouseInput { .. },
                            ..
                        } => true,
                        winit::event::Event::WindowEvent {
                            event: winit::event::WindowEvent::MouseWheel { .. },
                            ..
                        } => true,
                        _ => false,
                    };

                input_captured
            };

            // if imgui didn't want the event, hand it off to the game
            if !input_captured {
                let mut input_state = resources.get_mut::<InputResource>().unwrap();
                let _app_control = resources.get_mut::<AppControlResource>().unwrap();
                let viewport = resources.get::<ViewportResource>().unwrap();
                minimum_winit::input::handle_winit_event(&event, &mut *input_state, &*viewport);
            }

            // Handle general update/redraw events
            match event {
                winit::event::Event::MainEventsCleared => {
                    {
                        let _imgui_manager =
                            resources.get_mut::<WinitImguiManagerResource>().unwrap();
                        winit_imgui_manager.begin_frame(&winit_window);
                    }
                    app_handler.update(&mut world, &mut resources);

                    // Queue a RedrawRequested event.
                    winit_window.request_redraw();

                    {
                        let imgui_manager =
                            resources.get_mut::<WinitImguiManagerResource>().unwrap();
                        imgui_manager.render(&winit_window);
                    }
                }
                winit::event::Event::RedrawRequested(_window_id) => {
                    let _imgui_manager = resources
                        .get::<WinitImguiManagerResource>()
                        .unwrap()
                        .clone();
                    let skulpin_window = skulpin::WinitWindow::new(&winit_window);
                    if let Err(e) =
                        renderer.draw(&skulpin_window, |canvas, coordinate_system_helper| {
                            resources
                                .get_mut::<CanvasDrawResource>()
                                .unwrap()
                                .begin_draw_context(canvas, coordinate_system_helper);
                            app_handler.draw(&mut world, &mut resources);
                            resources
                                .get_mut::<CanvasDrawResource>()
                                .unwrap()
                                .end_draw_context();
                        })
                    {
                        log::warn!("Passing Renderer::draw() error to app {}", e);
                        app_handler.fatal_error(&e.into());
                        {
                            let mut app_control =
                                resources.get_mut::<AppControlResource>().unwrap();
                            app_control.enqueue_terminate_process();
                        }
                    }
                }
                _ => {}
            }

            // Always check if we should terminate the application
            {
                let app_control = resources.get::<AppControlResource>().unwrap();
                if app_control.should_terminate_process() {
                    // Drop entities now as resources will be dropped
                    // physics components may point back at resources
                    world.delete_all();
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }
        });
    }
}

fn init_imgui(window: &winit::window::Window) -> imgui::Context {
    use imgui::Context;

    let mut imgui = Context::create();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = 1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let hidpi_factor = window.scale_factor().round();
    let font_size = (16.0 * hidpi_factor) as f32;

    let font_1p = imgui::FontSource::TtfData {
        data: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../fonts/mplus-1p-regular.ttf"
        )),
        size_pixels: font_size,
        config: None,
    };

    // Feather icons
    let font_feather = {
        const ICON_GLYPH_RANGE_FEATHER: [u16; 3] = [0xe81b, 0xe92a, 0];
        let mut font_config = imgui::FontConfig::default();
        font_config.glyph_ranges = imgui::FontGlyphRanges::from_slice(&ICON_GLYPH_RANGE_FEATHER);

        imgui::FontSource::TtfData {
            data: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fonts/feather.ttf")),
            size_pixels: font_size,
            config: Some(font_config),
        }
    };

    let font_material = {
        // Material icons
        const ICON_GLYPH_RANGE_MATERIAL: [u16; 13] = [
            //0xfd24, 0xfd34, // transform/rotate icons
            0xf3e4, 0xf3e4, // pause
            0xf40a, 0xf40a, // play
            0xf1b5, 0xf1b5, // select
            0xfd25, 0xfd25, // translate
            0xfd74, 0xfd74, // rotate
            0xfa67, 0xfa67, // scale
            0,
        ];
        let mut font_config = imgui::FontConfig::default();
        font_config.glyph_ranges = imgui::FontGlyphRanges::from_slice(&ICON_GLYPH_RANGE_MATERIAL);
        font_config.glyph_offset = [0.0, 6.0];
        font_config.glyph_min_advance_x = 16.0;

        imgui::FontSource::TtfData {
            data: include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../fonts/materialdesignicons-webfont.ttf"
            )),
            size_pixels: font_size,
            config: Some(font_config),
        }
    };

    imgui
        .fonts()
        .add_font(&[font_1p, font_feather, font_material]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    return imgui;
}

pub fn init_winit_imgui_manager(window: &winit::window::Window) -> WinitImguiManager {
    let mut imgui_context = init_imgui(&window);
    let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    imgui_platform.attach_window(
        imgui_context.io_mut(),
        window,
        imgui_winit_support::HiDpiMode::Locked(window.scale_factor()),
    );

    WinitImguiManager::new(imgui_context, imgui_platform)
}
