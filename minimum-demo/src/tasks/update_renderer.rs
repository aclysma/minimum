use crate::renderer::Renderer;
use crate::base::task::ReadAllTaskImpl;
use crate::base::ResourceMap;
use crate::base::TaskConfig;
use crate::base::TaskContextFlags;
use rendy::wsi::winit;

pub struct UpdateRenderer;
pub type UpdateRendererTask = crate::base::ReadAllTask<UpdateRenderer>;
impl ReadAllTaskImpl for UpdateRenderer {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhaseRender>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &ResourceMap) {
        let window = resource_map.fetch::<winit::window::Window>();
        let mut renderer = resource_map.fetch_mut::<Renderer>();
        renderer.render(&window, &resource_map);
    }
}
