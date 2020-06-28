#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod math;

pub mod matrix;

pub use math::Vec2;
pub use math::Vec3;
pub use math::Vec4;

pub use matrix::Mat4;
