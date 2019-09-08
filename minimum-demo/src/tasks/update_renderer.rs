
use rendy::wsi::winit;
use minimum::task::ReadAllTaskImpl;
use minimum::TaskConfig;
use minimum::ResourceMap;
use crate::resources;

use named_type::NamedType;

#[derive(NamedType)]
pub struct UpdateRenderer;
pub type UpdateRendererTask = minimum::ReadAllTask<UpdateRenderer>;
impl ReadAllTaskImpl for UpdateRenderer {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhaseRender>();
    }

    fn run(resource_map: &ResourceMap) {
        let window = resource_map.fetch::<winit::window::Window>();
        let mut renderer = resource_map.fetch_mut::<crate::renderer::Renderer>();
        renderer.render(&window, &resource_map);
    }
}
