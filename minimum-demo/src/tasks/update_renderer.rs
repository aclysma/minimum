
use rendy::wsi::winit;
use minimum::task::ReadAllTaskImpl;
use minimum::TaskConfig;
use minimum::ResourceMap;
use minimum::TaskContextFlags;
use crate::renderer::Renderer;

pub struct UpdateRenderer;
pub type UpdateRendererTask = minimum::ReadAllTask<UpdateRenderer>;
impl ReadAllTaskImpl for UpdateRenderer {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseRender>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &ResourceMap) {
        let window = resource_map.fetch::<winit::window::Window>();
        let mut renderer = resource_map.fetch_mut::<Renderer>();
        renderer.render(&window, &resource_map);
    }
}
