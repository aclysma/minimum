use minimum::component::SlabComponentStorage;
use minimum::Component;

#[derive(Clone)]
pub struct EditorModifiedComponent {}

impl EditorModifiedComponent {
    pub fn new() -> Self {
        EditorModifiedComponent {}
    }
}

impl Component for EditorModifiedComponent {
    type Storage = SlabComponentStorage<Self>;
}
