use legion::prelude::World;
use minimum_math::Segment;
use minimum_math::NormalizedRay;

pub struct ViewportResource {
    // Size of the viewport in pixels
    size_in_pixels: glam::Vec2,

    // Resolution-independent 2D view. This is useful for screen-space drawing that scales
    // proportionally with the viewport size in pixels
    screen_space_matrix: glam::Mat4,
    screen_space_matrix_inv: glam::Mat4,

    // 3D world space
    world_space_proj_matrix: glam::Mat4,
    world_space_proj_matrix_inv: glam::Mat4,
    world_space_view_matrix: glam::Mat4,
    world_space_view_matrix_inv: glam::Mat4,

    // Normalized vector
    world_space_eye_position: glam::Vec3,
    // world_space_eye_direction: glam::Vec3,
    // world_space_up: glam::Vec3,
    // fov: f32, // 0.0 = ortho?
    // near_clip: f32,
    // far_clip: f32,
}

impl ViewportResource {
    pub fn empty() -> Self {
        ViewportResource {
            size_in_pixels: glam::Vec2::zero(),
            screen_space_matrix: glam::Mat4::zero(),
            screen_space_matrix_inv: glam::Mat4::zero(),
            world_space_proj_matrix: glam::Mat4::zero(),
            world_space_proj_matrix_inv: glam::Mat4::zero(),
            world_space_view_matrix: glam::Mat4::zero(),
            world_space_view_matrix_inv: glam::Mat4::zero(),
            world_space_eye_position: glam::Vec3::zero(),
            // world_space_eye_direction: glam::Vec3::zero(),
            // world_space_up: glam::Vec3::zero(),
            // fov: 0.0,
            // near_clip: 0.0,
            // far_clip: 0.0,

        }
    }

    pub fn world_space_eye_position(&self) -> glam::Vec3 {
        self.world_space_eye_position
    }

    pub fn screen_space_matrix(&self) -> &glam::Mat4 {
        &self.screen_space_matrix
    }

    pub fn set_viewport_size_in_pixels(&mut self, size_in_pixels: glam::Vec2) {
        self.size_in_pixels = size_in_pixels;
    }

    pub fn set_screen_space_view(
        &mut self,
        matrix: glam::Mat4,
    ) {
        self.screen_space_matrix = matrix;
        self.screen_space_matrix_inv = matrix.inverse();
    }

    pub fn set_world_space_view(
        &mut self,
        proj_matrix: glam::Mat4,
        view_matrix: glam::Mat4,
        eye: glam::Vec3,
        // dir: glam::Vec3,
        // up: glam::Vec3,
        // fov: f32,
        // near_clip: f32,
        // far_clip: f32,
    ) {
        self.world_space_proj_matrix = proj_matrix;
        self.world_space_proj_matrix_inv = proj_matrix.inverse();
        self.world_space_view_matrix = view_matrix;
        self.world_space_view_matrix_inv = view_matrix.inverse();
        self.world_space_eye_position = eye;
        // self.world_space_eye_direction = dir;
        // self.world_space_up = up;
        // self.fov = fov;
        // self.near_clip = near_clip;
        // self.far_clip = far_clip;
    }

    // In: the normalized device coordinates. Top left: (-1, -1) Bottom right: (1, 1)
    // Out: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    pub fn normalized_space_to_viewport_space(
        &self,
        normalized_position: glam::Vec2,
    ) -> glam::Vec2 {
        //(viewport_position * self.size_in_pixels * glam::Vec2::new(0.5, 0.5)) + self.size_in_pixels
        ((normalized_position + glam::Vec2::new(1.0, 1.0)) * self.size_in_pixels * glam::Vec2::new(0.5, 0.5))
    }

    // In: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    // Out: the normalized device coordinates. Top left: (-1, -1) Bottom right: (1, 1)
    pub fn viewport_space_to_normalized_space(
        &self,
        viewport_position: glam::Vec2,
    ) -> glam::Vec2 {
        ((viewport_position * glam::Vec2::new(2.0, 2.0) / self.size_in_pixels) - glam::Vec2::new(1.0, 1.0)) //* (glam::Vec2::new(1.0, -1.0))
    }

    // In: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    // Out: Position in world space. If clip range = 0 it will intersect the near plane and if
    //      clip range = 1 it will intersect the far plane
    pub fn viewport_space_to_world_space(
        &self,
        viewport_position: glam::Vec2,
        clip_range: f32
    ) -> glam::Vec3 {
        let position = self.viewport_space_to_normalized_space(viewport_position);
        let mut position = glam::Vec4::new(position.x(), position.y(), clip_range, 1.0);

        // project to view space
        position = self.world_space_proj_matrix_inv * position;

        // project to world space
        position = self.world_space_view_matrix_inv * position;

        position = position / position.w();

        position.truncate()
    }

    pub fn viewport_space_to_segment(
        &self,
        viewport_position: glam::Vec2,
    ) -> Segment {
        Segment {
            p0: self.viewport_space_to_world_space(viewport_position, 0.0),
            p1: self.viewport_space_to_world_space(viewport_position, 1.0),
        }
    }

    pub fn viewport_space_to_ray(
        &self,
        viewport_position: glam::Vec2,
    ) -> NormalizedRay {
        let segment = self.viewport_space_to_segment(viewport_position);

        let mut dir = segment.p1 - segment.p0;
        let length = dir.length();
        dir = dir / length;

        NormalizedRay {
            origin: segment.p0,
            dir,
            length
        }
    }

    // In: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    // Out: Position in view space. z will be -1.0, x/y will be an angle offset from center based
    //      on fov.
    // NOTE: Unlike viewport_space_to_world_space we do not bother with a clip_range. The only thing
    //       it would change is the w component, and the w component is only useful for getting
    //       world space. Not sure if this function is particularly useful to be honest.
    pub fn viewport_space_to_view_space(
        &self,
        viewport_position: glam::Vec2,
    ) -> glam::Vec3 {
        let position = self.viewport_space_to_normalized_space(viewport_position);
        let mut position = glam::Vec4::new(position.x(), position.y(), 0.0, 1.0);

        // project to view space
        position = self.world_space_proj_matrix_inv * position;

        position.truncate()
    }

    // In: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    // Out: Position in screen space.
    // NOTE: Unlike viewport_space_to_world_space we do not bother with a clip_range. This
    //       projection is presumed to be orthographic and I don't think we need to do a correction
    //       using the w component
    pub fn viewport_space_to_screen_space(
        &self,
        viewport_position: glam::Vec2,
    ) -> glam::Vec2 {
        let position = self.viewport_space_to_normalized_space(viewport_position);
        let mut position = glam::Vec4::new(position.x(), position.y(), 0.0, 1.0);

        // project to world space
        let position = self.screen_space_matrix_inv * position;

        glam::Vec2::new(position.x(), position.y())
    }

    // In: Position in world space.
    // Out: pixel coordinates within the viewport. Top left: (0, 0) Bottom right: self.size_in_pixels
    pub fn world_space_to_viewport_space(
        &self,
        world_position: glam::Vec3,
    ) -> glam::Vec2 {
        // input is a position in pixels
        let mut position = glam::Vec4::new(world_position.x(), world_position.y(), world_position.z(), 1.0);

        // project to raw space
        position = self.world_space_proj_matrix * self.world_space_view_matrix * position;
        position = position / position.w();

        // project to world space
        self.normalized_space_to_viewport_space(glam::Vec2::new(position.x(), position.y()))
    }

    pub fn viewport_space_delta_to_world_space_delta(
        &self,
        viewport_space_delta: glam::Vec2,
    ) -> glam::Vec3 {
        // Find the world space delta
        let world_space_zero = self.viewport_space_to_world_space(glam::Vec2::zero(), 0.0);
        self.viewport_space_to_world_space(viewport_space_delta, 0.0) - world_space_zero
    }


    pub fn world_space_clip(
        &self,
        world_position: glam::Vec3,
    ) -> f32 {
        // input is a position in pixels
        let mut position = world_position.extend(1.0);

        // project to raw space
        position = self.world_space_proj_matrix * self.world_space_view_matrix * position;
        position = position / position.w();

        position.z()
    }

    // I'm sure there's a much better way to do this, but it looks good and is consistent across FOV
    // unlike just using distance. I'm not even sure exactly what it returns, but treat it like a
    // unit-less ratio and scale world-space things by it.
    pub fn world_space_ui_multiplier(
        &self,
        position: glam::Vec3
    ) -> f32 {
        let position_2d = self.world_space_to_viewport_space(position);
        let depth = self.world_space_clip(position);

        // Determine the world space distance we would get if we translate in screen space
        let offset_min = self.viewport_space_to_world_space(position_2d - glam::Vec2::new(50.0, 50.0), depth);
        let offset_max = self.viewport_space_to_world_space(position_2d + glam::Vec2::new(50.0, 50.0), depth);
        (offset_max - offset_min).length()
    }

    /*
    pub fn apply_viewport_delta_to_world_space(
        &self,
        world_space: glam::Vec3,
        viewport_space_delta: glam::Vec2,
    ) -> glam::Vec3 {
        // input is a position in pixels
        let mut position = world_space.extend(1.0);

        // project to raw space
        position = self.world_space_proj_matrix * self.world_space_view_matrix * position;
        position = position / position.w();

        let z = position.z();

        // project to world space
        let mut viewport_space = self.normalized_space_to_viewport_space(glam::Vec2::new(position.x(), position.y()));
        viewport_space += viewport_space_delta;

        self.viewport_space_to_world_space(viewport_space, z)

        // let normalized = self.viewport_space_to_normalized_space(viewport_space);
        //
        // let position = glam::Vec4::new(normalized.x(), normalized.y(), z, 1.0);
        // position =
    }
    */
}
