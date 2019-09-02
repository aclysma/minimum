
use imgui_inspect::InspectRenderDefault;
use imgui_inspect::InspectArgsDefault;

pub struct ImGlmVec4;
impl InspectRenderDefault<glm::Vec4> for ImGlmVec4 {
    fn render(
        data: &[&glm::Vec4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        ui.text(&imgui::im_str!(
            "{}: {} {} {} {}",
            label,
            data[0].x,
            data[0].y,
            data[0].z,
            data[0].w
        ));
    }

    fn render_mut(
        data: &mut [&mut glm::Vec4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x, data[0].y, data[0].z, data[0].w];
        if ui
            .input_float4(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.x = val[0];
                d.y = val[1];
                d.z = val[2];
                d.w = val[3];
            }
        }

        changed
    }
}