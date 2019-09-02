use minimum::component::SlabComponentStorage;
use minimum::Component;
use named_type::NamedType;

#[derive(Clone, NamedType)]
pub struct EditorSelectedComponent {}

impl EditorSelectedComponent {
    pub fn new() -> Self {
        EditorSelectedComponent {}
    }
}

impl Component for EditorSelectedComponent {
    type Storage = SlabComponentStorage<Self>;
}
