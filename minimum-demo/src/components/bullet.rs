use minimum::component::SlabComponentStorage;
use named_type::NamedType;

// This component contains no data, however an empty component can still be useful to "tag" entities
#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
pub struct BulletComponent {}

impl BulletComponent {
    pub fn new() -> Self {
        BulletComponent {}
    }
}

impl minimum::Component for BulletComponent {
    type Storage = SlabComponentStorage<Self>;
}
