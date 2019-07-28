//TODO: Use this rather than the queue in entity_set

use crate::component::SlabComponentStorage;
use crate::Component;

#[derive(Debug)]
pub struct PendingDeleteComponent {}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        PendingDeleteComponent {}
    }
}

impl Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<PendingDeleteComponent>;
}
