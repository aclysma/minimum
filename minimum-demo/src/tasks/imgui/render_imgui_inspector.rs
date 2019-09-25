use crate::resources;
use crate::base::task::WriteAllTaskImpl;
use crate::base::ResourceMap;
use crate::base::TaskConfig;
use crate::base::TaskContextFlags;

pub struct RenderImguiInspector;
pub type RenderImguiInspectorTask = crate::base::WriteAllTask<RenderImguiInspector>;
impl WriteAllTaskImpl for RenderImguiInspector {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePreRender>();
    }

    fn run(_context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        let mut imgui_manager = resource_map.fetch_mut::<resources::ImguiManager>();

        //TODO: draw_inspector could potentially take a <T: ImguiManager> param
        imgui_manager.with_ui(|ui| {
            crate::framework::inspect::draw_inspector(&resource_map, ui);
        });
    }
}
