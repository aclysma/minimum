use skulpin::app::LogicalSize;

use std::ffi::CString;

use demo::DemoApp;
use minimum::daemon;

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("tokio_reactor", log::LevelFilter::Info)
        .init();

    // Spawn the daemon in a background thread. This could be a different process, but
    // for simplicity we'll launch it here.
    std::thread::spawn(move || {
        daemon::run();
    });

    // Build the app and run it
    let example_app = DemoApp::new();
    let renderer_builder = skulpin::RendererBuilder::new()
        .app_name(CString::new("Skulpin Example App").unwrap())
        .use_vulkan_debug_layer(true);

    demo::app::App::run(example_app, LogicalSize::new(900, 600), renderer_builder);
}
