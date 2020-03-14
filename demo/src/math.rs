use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;
use skulpin::app::PhysicalPosition;

pub fn vec2_glam_to_glm(value: glam::Vec2) -> glm::Vec2 {
    glm::Vec2::new(value.x(), value.y())
}

pub fn vec3_glam_to_glm(value: glam::Vec3) -> glm::Vec3 {
    glm::Vec3::new(value.x(), value.y(), value.z())
}

pub fn vec4_glam_to_glm(value: glam::Vec4) -> glm::Vec4 {
    glm::Vec4::new(value.x(), value.y(), value.z(), value.w())
}

pub fn vec2_glm_to_glam(value: glm::Vec2) -> glam::Vec2 {
    glam::Vec2::new(value.x, value.y)
}

pub fn vec3_glm_to_glam(value: glm::Vec3) -> glam::Vec3 {
    glam::Vec3::new(value.x, value.y, value.z)
}

pub fn vec4_glm_to_glam(value: glm::Vec4) -> glam::Vec4 {
    glam::Vec4::new(value.x, value.y, value.z, value.w)
}

pub fn winit_position_to_glam(position: PhysicalPosition<f64>) -> glam::Vec2 {
    glam::Vec2::new(position.x as f32, position.y as f32)
}

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

impl From<glm::Vec2> for Vec2 {
    fn from(value: glm::Vec2) -> Self {
        Vec2 {
            value: vec2_glm_to_glam(value),
        }
    }
}

impl Into<glm::Vec2> for Vec2 {
    fn into(self) -> glm::Vec2 {
        vec2_glam_to_glm(self.value)
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
            value: self.value.truncate()
        }
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

impl From<glm::Vec3> for Vec3 {
    fn from(value: glm::Vec3) -> Self {
        Vec3 {
            value: vec3_glm_to_glam(value),
        }
    }
}

impl Into<glm::Vec3> for Vec3 {
    fn into(self) -> glm::Vec3 {
        vec3_glam_to_glm(self.value)
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

impl From<glm::Vec4> for Vec4 {
    fn from(value: glm::Vec4) -> Self {
        Vec4 {
            value: vec4_glm_to_glam(value),
        }
    }
}

impl Into<glm::Vec4> for Vec4 {
    fn into(self) -> glm::Vec4 {
        vec4_glam_to_glm(self.value)
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
