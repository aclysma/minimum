use minimum::component::VecComponentStorage;
use minimum::component::DefaultComponentReflector;
use named_type::NamedType;
use crate::inspect::common_types::*;

#[derive(Debug, Clone, NamedType, imgui_inspect_derive::Inspect)]
pub struct PositionComponent {
    #[inspect(proxy_type = "ImGlmVec2", on_set = "inspect_position_updated")]
    position: glm::Vec2,


    //TODO: Use Skip
    //TODO: Find a better way to handle this
    //#[inspect(skip)]
    moved: bool
}

impl PositionComponent {
    pub fn new(position: glm::Vec2) -> Self {
        PositionComponent { position, moved: false }
    }

    pub fn position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.position
    }

    pub fn inspect_position_updated(&mut self) {
        println!("posiiton updated");
        self.moved = true;
    }
}

impl minimum::Component for PositionComponent {
    type Storage = VecComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}

use minimum::component::SlabComponentStorage;

#[derive(Debug, Clone, NamedType)]
pub struct PositionChangedComponent {
    position: glm::Vec2,
}


impl PositionChangedComponent {
    pub fn new(position: glm::Vec2) -> Self {
        PositionChangedComponent { position }
    }
}

impl minimum::Component for PositionChangedComponent {
    type Storage = SlabComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}