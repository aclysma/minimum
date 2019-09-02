
use imgui_inspect::InspectRenderDefault;
use imgui_inspect::InspectArgsDefault;

pub struct ImGlmColor4;
impl InspectRenderDefault<glm::Vec4> for ImGlmColor4 {
    fn render(
        data: &[&glm::Vec4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        ui.color_button(
            &imgui::im_str!("{}", label),
            [data[0].x, data[0].y, data[0].z, data[0].w],
        )
            .build();
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
            .color_edit(
                &imgui::im_str!("{}", label),
                imgui::EditableColor::from(&mut val),
            )
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
