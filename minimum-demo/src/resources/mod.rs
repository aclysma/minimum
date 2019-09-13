mod debug_draw;
mod debug_options;
mod input_manager;
mod physics_manager;
mod render_state;
mod window_interface;

pub use debug_draw::DebugDraw;
pub use debug_options::DebugOptions;
pub use input_manager::InputManager;
pub use input_manager::MouseButtons;
pub use physics_manager::PhysicsManager;
pub use render_state::RenderState;
pub use window_interface::WindowInterface;
pub use window_interface::WindowUserEvent;

#[cfg(feature = "editor")]
mod imgui_manager;
#[cfg(feature = "editor")]
pub use imgui_manager::ImguiManager;

#[cfg(not(feature = "editor"))]
pub struct ImguiManager;

#[cfg(not(feature = "editor"))]
impl ImguiManager {
    pub fn want_capture_keyboard(&self) -> bool {
        false
    }

    pub fn want_capture_mouse(&self) -> bool {
        false
    }

    pub fn want_set_mouse_pos(&self) -> bool {
        false
    }

    pub fn want_text_input(&self) -> bool {
        false
    }
}
