use crate::component::SlabComponentStorage;
use crate::Component;

/// This component is used internally to flag a component to be deleted later.
#[derive(Debug)]
pub struct PendingDeleteComponent {}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        PendingDeleteComponent {}
    }
}

impl Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<Self>;
}
