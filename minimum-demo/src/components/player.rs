#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use crate::base::component::SlabComponentStorage;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Inspect)]
pub struct PlayerComponent {}

impl PlayerComponent {
    pub fn new() -> Self {
        PlayerComponent {}
    }
}

impl crate::base::Component for PlayerComponent {
    type Storage = SlabComponentStorage<PlayerComponent>;
}
