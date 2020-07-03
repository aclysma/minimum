
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoundingSphere {
    pub center: glam::Vec3,
    pub radius: f32
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoundingAabb {
    pub min: glam::Vec3,
    pub max: glam::Vec3
}

impl BoundingAabb {
    pub fn new(initial_point: glam::Vec3) -> Self {
        BoundingAabb {
            min: initial_point,
            max: initial_point
        }
    }

    pub fn expand(&mut self, p: glam::Vec3) {
        self.max = self.max.max(p);
        self.min = self.min.min(p);
    }

    pub fn calculate_bounding_sphere(&self) -> BoundingSphere {
        let center = (self.min + self.max) / 2.0;
        let radius = (self.min - self.max).length() / 2.0;

        BoundingSphere {
            center,
            radius
        }
    }
}