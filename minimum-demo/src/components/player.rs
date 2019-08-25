use minimum::component::SlabComponentStorage;
use named_type::NamedType;
use imgui_inspect_derive::Inspect;

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
