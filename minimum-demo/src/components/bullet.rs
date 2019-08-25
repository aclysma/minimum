use minimum::component::SlabComponentStorage;
use named_type::NamedType;
use serde::{Serialize, Deserialize};
use imgui_inspect_derive::Inspect;

// This component contains no data, however an empty component can still be useful to "tag" entities
#[derive(Debug, Clone, NamedType, Inspect, Serialize, Deserialize)]
pub struct BulletComponent {}

impl BulletComponent {
    pub fn new() -> Self {
        BulletComponent {}
    }
}

impl minimum::Component for BulletComponent {
    type Storage = SlabComponentStorage<Self>;
}
