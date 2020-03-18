pub use example_shared::resources::*;

mod canvas_draw;
pub use canvas_draw::CanvasDrawResource;

mod sdl2_imgui;
pub use sdl2_imgui::Sdl2ImguiManagerResource;

mod sdl2_window;
pub use sdl2_window::Sdl2WindowResource;
