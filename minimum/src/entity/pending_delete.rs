//TODO: Use this rather than the queue in entity_set

use crate::component::SlabComponentStorage;
use crate::component::DefaultComponentReflector;
use crate::Component;

#[derive(Debug, typename::TypeName)]
pub struct PendingDeleteComponent {}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        PendingDeleteComponent {}
    }
}

impl Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
