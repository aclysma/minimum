use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ResourceTaskImpl, TaskConfig, TaskContextFlags};
use rendy::wsi::winit;

use crate::resources::ImguiManager;

pub struct ImguiBeginFrame;
pub type ImguiBeginFrameTask = minimum::ResourceTask<ImguiBeginFrame>;
impl ResourceTaskImpl for ImguiBeginFrame {
    type RequiredResources = (Read<winit::window::Window>, Write<ImguiManager>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseFrameBegin>();
        config.run_only_if(framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (window, mut imgui_manager) = data;
        imgui_manager.begin_frame(&window);
    }
}
