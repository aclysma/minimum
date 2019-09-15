
use minimum::component::SlabComponentStorage;
use minimum::Component;

#[derive(Clone)]
pub struct EditorTranslatedComponent {
    delta: glm::Vec2
}

impl EditorTranslatedComponent {
    pub fn new(delta: glm::Vec2) -> Self {
        EditorTranslatedComponent {
            delta
        }
    }
}

impl Component for EditorTranslatedComponent {
    type Storage = SlabComponentStorage<Self>;
}
