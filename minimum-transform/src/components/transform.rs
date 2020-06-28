use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use minimum_math::matrix::Mat4;

//
// Primary transform component, usually populated by using other components
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect, Default)]
#[uuid = "35657365-bb0c-4306-8c69-d5e158ad978f"]
pub struct LocalToWorldComponent {
    #[serde_diff(opaque)]
    pub transform: Mat4,
}

legion_prefab::register_component_type!(LocalToWorldComponent);
