use minimum::component::SlabComponentStorage;

#[derive(Debug, Clone)]
pub struct PlayerComponent {}

impl PlayerComponent {
    pub fn new() -> Self {
        PlayerComponent {}
    }
}

impl minimum::Component for PlayerComponent {
    type Storage = SlabComponentStorage<PlayerComponent>;
}