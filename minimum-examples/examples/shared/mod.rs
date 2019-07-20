pub mod components;
pub mod resources;
pub mod tasks;

#[derive(Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 {
            x,
            y
        }
    }
}