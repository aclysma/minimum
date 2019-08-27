use imgui_inspect_derive::Inspect;
use minimum::component::SlabComponentStorage;
use named_type::NamedType;

#[derive(Debug, Clone, NamedType, Inspect)]
pub struct PlayerComponent {}

impl PlayerComponent {
    pub fn new() -> Self {
        PlayerComponent {}
    }
}

impl minimum::Component for PlayerComponent {
    type Storage = SlabComponentStorage<PlayerComponent>;
}
