use minimum::component::{ComponentStorage, VecComponentStorage, DefaultComponentReflector};
use minimum::Component;
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::ResourceMap;

use nphysics2d::object::ColliderHandle;

use ncollide2d::shape::ShapeHandle;
use ncollide2d::world::{CollisionGroups, GeometricQueryType};
use std::collections::VecDeque;

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
