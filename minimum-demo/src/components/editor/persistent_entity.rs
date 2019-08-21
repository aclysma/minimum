
use minimum::component::SlabComponentStorage;
use minimum::Component;
use named_type::NamedType;
use crate::constructors::PersistentEntityPrototype;
use minimum::component::DefaultComponentReflector;

#[derive(NamedType, Clone)]
pub struct PersistentEntityComponent {
    prototype: PersistentEntityPrototype
}

impl PersistentEntityComponent {
    pub fn new(prototype: PersistentEntityPrototype) -> Self {
        PersistentEntityComponent {
            prototype
        }
    }

    pub fn prototype(&self) -> &PersistentEntityPrototype {
        &self.prototype
    }
}

impl Component for PersistentEntityComponent {
    type Storage = SlabComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
