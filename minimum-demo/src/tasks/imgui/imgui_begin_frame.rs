use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};
use rendy::wsi::winit;

use crate::resources::ImguiManager;

pub struct ImguiBeginFrame;
pub type ImguiBeginFrameTask = crate::base::ResourceTask<ImguiBeginFrame>;
impl ResourceTaskImpl for ImguiBeginFrame {
    type RequiredResources = (Read<winit::window::Window>, Write<ImguiManager>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseFrameBegin>();
        config.run_only_if(crate::framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (window, mut imgui_manager) = data;
        imgui_manager.begin_frame(&window);
    }
}
