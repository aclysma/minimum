pub use example_shared::resources::*;

mod canvas_draw;
pub use canvas_draw::CanvasDrawResource;

mod winit_imgui;
pub use self::winit_imgui::WinitImguiManagerResource;

mod winit_window;
pub use winit_window::WinitWindowResource;
