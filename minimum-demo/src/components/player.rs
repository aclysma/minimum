use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType)]
pub struct PlayerComponent {}

impl PlayerComponent {
    pub fn new() -> Self {
        PlayerComponent {}
    }
}

impl minimum::Component for PlayerComponent {
    type Storage = SlabComponentStorage<PlayerComponent>;
    type Reflector = DefaultComponentReflector<Self>;
}
