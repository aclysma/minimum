#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod math;

pub mod matrix;

pub use math::Vec2;
pub use math::Vec3;
pub use math::Vec4;
pub use math::Quat;

pub use matrix::Mat4;

pub mod functions;
pub use functions::Segment;
pub use functions::NormalizedRay;

pub mod bounds;
pub use bounds::BoundingSphere;
pub use bounds::BoundingAabb;

#[cfg(feature = "na_conversion")]
pub mod na_convert;
