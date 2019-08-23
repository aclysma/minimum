use minimum::component::DefaultComponentReflector;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
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
