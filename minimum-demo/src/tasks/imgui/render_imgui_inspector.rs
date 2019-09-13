use crate::resources;
use minimum::task::WriteAllTaskImpl;
use minimum::ResourceMap;
use minimum::TaskConfig;
use minimum::TaskContextFlags;

pub struct RenderImguiInspector;
pub type RenderImguiInspectorTask = minimum::WriteAllTask<RenderImguiInspector>;
impl WriteAllTaskImpl for RenderImguiInspector {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        let mut imgui_manager = resource_map.fetch_mut::<resources::ImguiManager>();

        //TODO: draw_inspector could potentially take a <T: ImguiManager> param
        imgui_manager.with_ui(|ui| {
            framework::inspect::draw_inspector(&resource_map, ui);
        });
    }
}
