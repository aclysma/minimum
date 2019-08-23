use crate::component::DefaultComponentReflector;
use crate::component::SlabComponentStorage;
use crate::Component;
use named_type::NamedType;

#[derive(Debug, NamedType)]
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
