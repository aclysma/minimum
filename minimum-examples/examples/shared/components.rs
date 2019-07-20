
use super::Vec2;

use minimum::component::Component;

#[derive(Debug)]
pub struct PositionComponent {
    pub position: Vec2,
}

impl PositionComponent {
    pub fn new(position: Vec2) -> Self {
        PositionComponent { position }
    }
}

impl Component for PositionComponent {
    type Storage = minimum::component::VecComponentStorage<PositionComponent>;
}

#[derive(Debug)]
pub struct VelocityComponent {
    pub velocity: Vec2,
}

impl VelocityComponent {
    pub fn new(velocity: Vec2) -> Self {
        VelocityComponent { velocity }
    }
}

impl Component for VelocityComponent {
    type Storage = minimum::component::SlabComponentStorage<VelocityComponent>;
}

#[derive(Debug)]
pub struct SpeedMultiplierComponent {
    pub multiplier: f32,
}

impl SpeedMultiplierComponent {
    pub fn new(multiplier: f32) -> Self {
        SpeedMultiplierComponent { multiplier }
    }
}

impl Component for SpeedMultiplierComponent {
    type Storage = minimum::component::SlabComponentStorage<SpeedMultiplierComponent>;
}