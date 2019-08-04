use minimum::component::{VecComponentStorage, DefaultComponentReflector};
use minimum::Component;

#[derive(Clone, typename::TypeName)]
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
