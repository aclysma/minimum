
use minimum::task::WriteAllTaskImpl;
use minimum::TaskConfig;
use minimum::ResourceMap;
use crate::resources;

pub struct RenderImguiInspector;
pub type RenderImguiInspectorTask = minimum::WriteAllTask<RenderImguiInspector>;
impl WriteAllTaskImpl for RenderImguiInspector {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePreRender>();
    }

    fn run(resource_map: &mut ResourceMap) {
        let mut imgui_manager = resource_map.fetch_mut::<resources::ImguiManager>();

        //TODO: draw_inspector could potentially take a <T: ImguiManager> param
        let ui = imgui_manager.with_ui(|ui| {
            framework::inspect::draw_inspector(&resource_map, ui);
        });
    }
}
