use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Mat4 {
    value: glam::Mat4,
}

impl Mat4 {
    pub fn zero() -> Self {
        Mat4 {
            value: glam::Mat4::zero(),
        }
    }
}

impl From<glam::Mat4> for Mat4 {
    fn from(value: glam::Mat4) -> Self {
        Mat4 { value }
    }
}

impl Into<glam::Mat4> for Mat4 {
    fn into(self) -> glam::Mat4 {
        *self
    }
}

impl Deref for Mat4 {
    type Target = glam::Mat4;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Mat4 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

fn render_row(
    ui: &imgui::Ui,
    label: &'static str,
    axis: &glam::Vec4,
) {
    ui.text(&imgui::im_str!(
        "{}: {} {} {} {}",
        label,
        axis.x(),
        axis.y(),
        axis.z(),
        axis.w()
    ));
}

fn render_row_mut<GetF, SetF>(
    ui: &imgui::Ui,
    label: &'static str,
    data: &mut [&mut Mat4],
    get_axis_fn: GetF,
    set_axis_fn: SetF,
) -> bool
where
    GetF: Fn(&mut glam::Mat4) -> glam::Vec4,
    SetF: Fn(&mut glam::Mat4, glam::Vec4),
{
    let mut changed = false;
    let axis = (get_axis_fn)(data[0]);
    let mut val = [axis.x(), axis.y(), axis.z(), axis.w()];
    if ui
        .input_float4(&imgui::im_str!("{}", label), &mut val)
        .build()
    {
        changed = true;
        let val: glam::Vec4 = val.into();
        for d in data {
            set_axis_fn(*d, val);
        }
    }

    changed
}

impl InspectRenderDefault<Mat4> for Mat4 {
    fn render(
        data: &[&Mat4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.is_empty() {
            return;
        }

        render_row(ui, label, &data[0].value.x_axis());
        render_row(ui, "", &data[0].value.y_axis());
        render_row(ui, "", &data[0].value.z_axis());
        render_row(ui, "", &data[0].value.w_axis());
    }

    fn render_mut(
        data: &mut [&mut Mat4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.is_empty() {
            return false;
        }

        let mut changed = false;
        changed |= render_row_mut(ui, label, data, |m| m.x_axis(), |m, v| m.set_x_axis(v));
        changed |= render_row_mut(ui, label, data, |m| m.y_axis(), |m, v| m.set_y_axis(v));
        changed |= render_row_mut(ui, label, data, |m| m.z_axis(), |m, v| m.set_z_axis(v));
        changed |= render_row_mut(ui, label, data, |m| m.w_axis(), |m, v| m.set_w_axis(v));
        changed
    }
}
