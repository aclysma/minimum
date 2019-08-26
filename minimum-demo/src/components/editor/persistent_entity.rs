use crate::framework::FrameworkEntityPrototype;
use minimum::component::SlabComponentStorage;
use minimum::Component;
use named_type::NamedType;

#[derive(NamedType, Clone)]
pub struct PersistentEntityComponent {
    prototype: FrameworkEntityPrototype,
}

impl PersistentEntityComponent {
    pub fn new(prototype: FrameworkEntityPrototype) -> Self {
        PersistentEntityComponent { prototype }
    }

    pub fn prototype(&self) -> &FrameworkEntityPrototype {
        &self.prototype
    }
}

impl Component for PersistentEntityComponent {
    type Storage = SlabComponentStorage<Self>;
}
