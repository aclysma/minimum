use minimum::resource::{DataRequirement, Read, Write};
use minimum::{Task, TaskContext};

use crate::resources::ImguiManager;

#[derive(typename::TypeName)]
pub struct ImguiBeginFrame;
impl Task for ImguiBeginFrame {
    type RequiredResources = (Read<winit::window::Window>, Write<ImguiManager>);
    const REQUIRED_FLAGS: usize = crate::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (window, mut imgui_manager) = data;
        imgui_manager.begin_frame(&window);
    }
}
