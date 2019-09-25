use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;

pub struct ImGlmQuat;
impl InspectRenderDefault<glm::Quat> for ImGlmQuat {
    fn render(
        data: &[&glm::Quat],
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
            data[0].coords.x,
            data[0].coords.y,
            data[0].coords.z,
            data[0].coords.w
        ));
    }

    fn render_mut(
        data: &mut [&mut glm::Quat],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].coords.x, data[0].coords.y, data[0].coords.z, data[0].coords.w];
        if ui
            .input_float4(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.coords.x = val[0];
                d.coords.y = val[1];
                d.coords.z = val[2];
                d.coords.w = val[3];
            }
        }

        changed
    }
}
