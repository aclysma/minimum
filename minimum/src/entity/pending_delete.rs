use crate::component::SlabComponentStorage;
use crate::Component;
use named_type::NamedType;

/// This component is used internally to flag a component to be deleted later.
#[derive(Debug, NamedType)]
pub struct PendingDeleteComponent {}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        PendingDeleteComponent {}
    }
}

impl Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<Self>;
}
