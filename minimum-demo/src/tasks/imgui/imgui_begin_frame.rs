use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ResourceTaskImpl, TaskConfig};
use rendy::wsi::winit;

use crate::resources::ImguiManager;

pub struct ImguiBeginFrame;
pub type ImguiBeginFrameTask = minimum::ResourceTask<ImguiBeginFrame>;
impl ResourceTaskImpl for ImguiBeginFrame {
    type RequiredResources = (Read<winit::window::Window>, Write<ImguiManager>);
    //const REQUIRED_FLAGS: usize = framework::context_flags::AUTHORITY_CLIENT as usize;

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseFrameBegin>();
    }

    fn run(
        //&mut self,
        //_task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (window, mut imgui_manager) = data;
        imgui_manager.begin_frame(&window);
    }
}
