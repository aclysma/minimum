use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Vec2 {
    value: glam::Vec2,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2 {
            value: glam::Vec2::zero(),
        }
    }
}

impl From<[f32;2]> for Vec2 {
    fn from(value: [f32;2]) -> Self {
        Vec2 { value: glam::Vec2::new(value[0], value[1] ) }
    }
}

impl From<glam::Vec2> for Vec2 {
    fn from(value: glam::Vec2) -> Self {
        Vec2 { value }
    }
}

impl Into<glam::Vec2> for Vec2 {
    fn into(self) -> glam::Vec2 {
        *self
    }
}

impl Deref for Vec2 {
    type Target = glam::Vec2;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Vec2 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl InspectRenderDefault<Vec2> for Vec2 {
    fn render(
        data: &[&Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        ui.text(&imgui::im_str!(
            "{}: {} {}",
            label,
            data[0].x(),
            data[0].y()
        ));
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x(), data[0].y()];
        if ui
            .input_float2(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.set_x(val[0]);
                d.set_y(val[1]);
            }
        }

        changed
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Vec3 {
    value: glam::Vec3,
}

impl Vec3 {
    pub fn zero() -> Self {
        Vec3 {
            value: glam::Vec3::zero(),
        }
    }

    pub fn xy(&self) -> Vec2 {
        Vec2 {
            value: self.value.truncate(),
        }
    }
}

impl From<[f32;3]> for Vec3 {
    fn from(value: [f32;3]) -> Self {
        Vec3 { value: glam::Vec3::new(value[0], value[1], value[2] ) }
    }
}

impl From<glam::Vec3> for Vec3 {
    fn from(value: glam::Vec3) -> Self {
        Vec3 { value }
    }
}

impl Into<glam::Vec3> for Vec3 {
    fn into(self) -> glam::Vec3 {
        *self
    }
}

impl Deref for Vec3 {
    type Target = glam::Vec3;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Vec3 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl InspectRenderDefault<Vec3> for Vec3 {
    fn render(
        data: &[&Vec3],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        ui.text(&imgui::im_str!(
            "{}: {} {} {}",
            label,
            data[0].x(),
            data[0].y(),
            data[0].z()
        ));
    }

    fn render_mut(
        data: &mut [&mut Vec3],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x(), data[0].y(), data[0].z()];
        if ui
            .input_float3(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.set_x(val[0]);
                d.set_y(val[1]);
                d.set_z(val[2]);
            }
        }

        changed
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Vec4 {
    value: glam::Vec4,
}

impl Vec4 {
    pub fn zero() -> Self {
        Vec4 {
            value: glam::Vec4::zero(),
        }
    }
}

impl From<[f32;4]> for Vec4 {
    fn from(value: [f32;4]) -> Self {
        Vec4 { value: glam::Vec4::new(value[0], value[1], value[2], value[3] ) }
    }
}

impl From<glam::Vec4> for Vec4 {
    fn from(value: glam::Vec4) -> Self {
        Vec4 { value }
    }
}

impl Into<glam::Vec4> for Vec4 {
    fn into(self) -> glam::Vec4 {
        *self
    }
}

impl Deref for Vec4 {
    type Target = glam::Vec4;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Vec4 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl InspectRenderDefault<Vec4> for Vec4 {
    fn render(
        data: &[&Vec4],
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
            data[0].x(),
            data[0].y(),
            data[0].z(),
            data[0].w()
        ));
    }

    fn render_mut(
        data: &mut [&mut Vec4],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].x(), data[0].y(), data[0].z(), data[0].w()];
        if ui
            .input_float4(&imgui::im_str!("{}", label), &mut val)
            .build()
        {
            changed = true;
            for d in data {
                d.set_x(val[0]);
                d.set_y(val[1]);
                d.set_z(val[2]);
                d.set_w(val[3]);
            }
        }

        changed
    }
}
