use minimum::component::SlabComponentStorage;
use minimum::Component;

#[derive(Clone)]
pub struct EditorSelectedComponent {}

impl EditorSelectedComponent {
    pub fn new() -> Self {
        EditorSelectedComponent {}
    }
}

impl Component for EditorSelectedComponent {
    type Storage = SlabComponentStorage<Self>;
}
