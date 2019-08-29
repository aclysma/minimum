use minimum::component::SlabComponentStorage;
use minimum::Component;
use named_type::NamedType;

#[derive(Clone, NamedType)]
pub struct EditorModifiedComponent {}

impl EditorModifiedComponent {
    pub fn new() -> Self {
        EditorModifiedComponent {}
    }
}

impl Component for EditorModifiedComponent {
    type Storage = SlabComponentStorage<Self>;
}
