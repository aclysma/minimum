use type_uuid::TypeUuid;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use imgui_inspect_derive::Inspect;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Default, Inspect)]
#[uuid = "9dfad44f-72e8-4ba6-b89a-96b017fb9cd9"]
pub struct EditorMetadataComponent {
    pub name: String
}

legion_prefab::register_component_type!(EditorMetadataComponent);
