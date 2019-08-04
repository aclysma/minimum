use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;

#[derive(Debug, Clone, typename::TypeName)]
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
