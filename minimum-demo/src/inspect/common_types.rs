
use imgui_inspect::InspectRenderDefault;
use imgui_inspect::InspectRenderSlider;
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectArgsSlider;

pub struct ImGlmVec2;
impl InspectRenderDefault<glm::Vec2> for ImGlmVec2 {
    fn render(data: &[&glm::Vec2], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
        if (data.len() == 0) {
            return;
        }

        ui.text(&imgui::im_str!("{}: {} {}", label, data[0].x, data[0].y));
    }

    fn render_mut(data: &mut [&mut glm::Vec2], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) -> bool {
        if (data.len() == 0) {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x, data[0].y];
        if ui.input_float2(&imgui::im_str!("{}", label), &mut val).build() {
            changed = true;
            for d in data {
                d.x = val[0];
                d.y = val[1];
            }
        }

        changed
    }
}

pub struct ImGlmVec3;
impl InspectRenderDefault<glm::Vec3> for ImGlmVec3 {
    fn render(data: &[&glm::Vec3], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
        if (data.len() == 0) {
            return;
        }

        ui.text(&imgui::im_str!("{}: {} {} {}", label, data[0].x, data[0].y, data[0].z));
    }

    fn render_mut(data: &mut [&mut glm::Vec3], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) -> bool {
        if (data.len() == 0) {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x, data[0].y, data[0].z];
        if ui.input_float3(&imgui::im_str!("{}", label), &mut val).build() {
            changed = true;
            for d in data {
                d.x = val[0];
                d.y = val[1];
                d.z = val[2];
            }
        }

        changed
    }
}