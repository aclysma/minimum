use base::component::SlabComponentStorage;
use base::Component;

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
