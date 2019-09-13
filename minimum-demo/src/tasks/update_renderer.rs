use crate::renderer::Renderer;
use minimum::task::ReadAllTaskImpl;
use minimum::ResourceMap;
use minimum::TaskConfig;
use minimum::TaskContextFlags;
use rendy::wsi::winit;

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
