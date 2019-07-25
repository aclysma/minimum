use crate::resources::{ImguiManager, WindowUserEvent};
use imgui;

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
    let hidpi_factor = window.hidpi_factor().round();
    let font_size = (16.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/mplus-1p-regular.ttf"
        )),
        size_pixels: font_size,
        config: None,
    }]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    return imgui;
}

pub fn init_imgui_manager(world: &minimum::systems::World) -> ImguiManager {
    let window = world.fetch::<winit::window::Window>();
    let mut imgui_context = init_imgui(&window);
    let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
    imgui_platform.attach_window(
        imgui_context.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Rounded,
    );

    ImguiManager::new(imgui_context, imgui_platform)
}

pub fn create_window_interface(
    world: &mut minimum::systems::World,
    event_loop: &winit::event_loop::EventLoop<WindowUserEvent>,
) -> std::sync::mpsc::Sender<winit::event::Event<WindowUserEvent>> {
    let (winit_event_tx, winit_event_rx) =
        std::sync::mpsc::channel::<winit::event::Event<crate::resources::WindowUserEvent>>();

    let event_loop_proxy = event_loop.create_proxy();

    let window_interface = crate::resources::WindowInterface::new(
        std::sync::Mutex::new(winit_event_rx),
        event_loop_proxy,
    );
    world.insert(window_interface);

    winit_event_tx
}

pub fn create_renderer(world: &minimum::systems::World) -> crate::renderer::Renderer {
    let window = world.fetch::<winit::window::Window>();
    let window = &*window;

    let mut renderer = crate::renderer::Renderer::new();
    renderer.init_render_graph(&window, &world);
    renderer
}
