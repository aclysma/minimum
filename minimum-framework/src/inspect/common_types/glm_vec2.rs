use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;

pub struct ImGlmVec2;
impl InspectRenderDefault<glm::Vec2> for ImGlmVec2 {
    fn render(
        data: &[&glm::Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        ui.text(&imgui::im_str!("{}: {} {}", label, data[0].x, data[0].y));
    }

    fn render_mut(
        data: &mut [&mut glm::Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x, data[0].y];
        if ui
            .input_float2(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.x = val[0];
                d.y = val[1];
            }
        }

        changed
    }
}
