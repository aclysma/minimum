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
