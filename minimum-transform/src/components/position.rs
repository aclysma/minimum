use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use minimum_math::math::Vec3;

//
// 2D Position
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107194cf0"]
pub struct PositionComponent {
    #[serde_diff(opaque)]
    pub position: Vec3,
}

legion_prefab::register_component_type!(PositionComponent);
