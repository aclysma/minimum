pub struct CameraResource {
    pub position: glam::Vec2,
    pub x_half_extents: f32,
}

impl CameraResource {
    pub fn new(
        position: glam::Vec2,
        x_half_extents: f32,
    ) -> Self {
        CameraResource {
            position,
            x_half_extents,
        }
    }
}
