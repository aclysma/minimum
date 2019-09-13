use rendy::wsi::winit;

#[cfg(feature = "editor")]
use crate::imgui_themes;
#[cfg(feature = "editor")]
use crate::resources::ImguiManager;
#[cfg(feature = "editor")]
use imgui;

use crate::resources::WindowUserEvent;

#[cfg(feature = "editor")]
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
        imgui_themes::custom_theme(style);

        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let hidpi_factor = window.hidpi_factor().round();
    let font_size = (16.0 * hidpi_factor) as f32;

    let font_1p = imgui::FontSource::TtfData {
        data: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/mplus-1p-regular.ttf"
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
            data: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/feather.ttf")),
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
                "/assets/materialdesignicons-webfont.ttf"
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

#[cfg(feature = "editor")]
pub fn init_imgui_manager(resource_map: &minimum::resource::ResourceMap) -> ImguiManager {
    let window = resource_map.fetch::<winit::window::Window>();
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
    resource_map: &mut minimum::resource::ResourceMap,
    event_loop: &winit::event_loop::EventLoop<WindowUserEvent>,
) -> std::sync::mpsc::Sender<winit::event::Event<WindowUserEvent>> {
    let (winit_event_tx, winit_event_rx) =
        std::sync::mpsc::channel::<winit::event::Event<crate::resources::WindowUserEvent>>();

    let event_loop_proxy = event_loop.create_proxy();

    let window_interface = crate::resources::WindowInterface::new(
        std::sync::Mutex::new(winit_event_rx),
        event_loop_proxy,
    );
    resource_map.insert(window_interface);

    winit_event_tx
}

pub fn create_renderer(resource_map: &minimum::resource::ResourceMap) -> crate::renderer::Renderer {
    let window = resource_map.fetch::<winit::window::Window>();
    let window = &*window;

    let mut renderer = crate::renderer::Renderer::new();
    renderer.init_render_graph(&window, &resource_map);
    renderer
}
