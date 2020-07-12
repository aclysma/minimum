use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use minimum_math::Vec3;

//
// 2D Rotation
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect, Default)]
#[uuid = "6841f13d-fe38-4320-a8f8-1a6133f45e33"]
pub struct Rotation2DComponent {
    pub rotation: f32,
}

legion_prefab::register_component_type!(Rotation2DComponent);

//
// Primary rotation component, usually populated by using other components
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect, Default)]
#[uuid = "812a5a11-a364-43f1-81ca-168af943c411"]

pub struct RotationComponent {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32
}

impl RotationComponent {
    pub fn to_quat(&self) -> glam::Quat {
        // Default is rotating around y, x, z (i.e. y is up)
        // We are z-up, so sandwich with an X axis rotation. This is temporary until I can do
        // a better rotation system
        glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2) *
            glam::Quat::from_rotation_ypr(self.yaw, self.pitch, self.roll) *
            glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
    }
}

legion_prefab::register_component_type!(RotationComponent);
