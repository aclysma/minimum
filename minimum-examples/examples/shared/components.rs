use super::Vec2;

use minimum::component::Component;
use minimum::component::VecComponentStorage;
use minimum::component::SlabComponentStorage;
use minimum::component::DefaultComponentReflector;

#[derive(Debug, typename::TypeName)]
pub struct PositionComponent {
    pub position: Vec2,
}

impl PositionComponent {
    pub fn new(position: Vec2) -> Self {
        PositionComponent { position }
    }
}

impl Component for PositionComponent {
    type Storage = VecComponentStorage<Self>;
    type Reflector = DefaultComponentReflector<Self>;
}

#[derive(Debug, typename::TypeName)]
pub struct VelocityComponent {
    pub velocity: Vec2,
}

impl VelocityComponent {
    pub fn new(velocity: Vec2) -> Self {
        VelocityComponent { velocity }
    }
}

impl Component for VelocityComponent {
    type Storage = SlabComponentStorage<VelocityComponent>;
    type Reflector = DefaultComponentReflector<Self>;
}

#[derive(Debug, typename::TypeName)]
pub struct SpeedMultiplierComponent {
    pub multiplier: f32,
}

impl SpeedMultiplierComponent {
    pub fn new(multiplier: f32) -> Self {
        SpeedMultiplierComponent { multiplier }
    }
}

impl Component for SpeedMultiplierComponent {
    type Storage = SlabComponentStorage<SpeedMultiplierComponent>;
    type Reflector = DefaultComponentReflector<Self>;
}
