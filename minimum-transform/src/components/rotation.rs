use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;

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

pub struct RotationComponent {}

legion_prefab::register_component_type!(RotationComponent);
