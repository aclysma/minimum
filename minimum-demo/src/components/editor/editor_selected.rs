use minimum::component::{VecComponentStorage, DefaultComponentReflector};
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
    //TODO: HashMap storage
    type Storage = VecComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}
