
pub struct ViewportResource {
    // Dimension of the drawable space of the viewport in pixels
    screen_space_dimensions: glam::Vec2,

    // This is measured in pixels. Mouse click positions would likely be encoded in UI space
    ui_space_matrix: glam::Mat4,

    // Resolution-independent 2D view. This is useful for screen-space drawing that scales
    // proportionally with the viewport size in pixels
    screen_space_matrix: glam::Mat4,

    // 3D world space
    world_space_matrix: glam::Mat4,
}

impl ViewportResource {
    pub fn empty() -> Self {
        ViewportResource {
            screen_space_dimensions: glam::Vec2::zero(),
            ui_space_matrix: glam::Mat4::zero(),
            screen_space_matrix: glam::Mat4::zero(),
            world_space_matrix: glam::Mat4::zero(),
        }
    }

    pub fn screen_space_dimensions(&self) -> glam::Vec2 {
        self.screen_space_dimensions
    }

    // pub fn screen_space_dimensions_mut(&mut self) -> &mut glam::Vec2 {
    //     &mut self.screen_space_dimensions
    // }

    pub fn ui_space_matrix(&self) -> &glam::Mat4 {
        &self.ui_space_matrix
    }

    // pub fn ui_space_matrix_mut(&mut self) -> &mut glam::Mat4 {
    //     &mut self.ui_space_matrix
    // }

    pub fn screen_space_matrix(&self) -> &glam::Mat4 {
        &self.screen_space_matrix
    }

    // pub fn screen_space_matrix_mut(&mut self) -> &mut glam::Mat4 {
    //     &mut self.screen_space_matrix
    // }

    pub fn world_space_matrix(&self) -> &glam::Mat4 {
        &self.world_space_matrix
    }

    // pub fn world_space_matrix_mut(&mut self) -> &mut glam::Mat4 {
    //     &mut self.world_space_matrix
    // }

    pub fn set_ui_space_view(
        &mut self,
        matrix: glam::Mat4,
    ) {
        self.ui_space_matrix = matrix;
    }

    pub fn set_screen_space_view(
        &mut self,
        matrix: glam::Mat4,
        dimensions: glam::Vec2,
    ) {
        self.screen_space_matrix = matrix;
        self.screen_space_dimensions = dimensions;
    }

    pub fn set_world_space_view(
        &mut self,
        //camera_position: glam::Vec3,
        matrix: glam::Mat4,
    ) {
        //self.world_space_camera_position = camera_position;
        self.world_space_matrix = matrix;
    }

    pub fn ui_space_to_world_space(
        &self,
        ui_position: glam::Vec2,
    ) -> glam::Vec3 {
        // input is a position in pixels
        let position = glam::Vec4::new(ui_position.x(), ui_position.y(), 0.0, 1.0);

        // project to raw space
        let position = self.ui_space_matrix * position;

        // project to world space
        let position = self.world_space_matrix.inverse() * position;

        glam::Vec3::new(position.x(), position.y(), position.z())
    }

    pub fn ui_space_to_screen_space(
        &self,
        ui_position: glam::Vec2,
    ) -> glam::Vec2 {
        // input is a position in pixels
        let position = glam::Vec4::new(ui_position.x(), ui_position.y(), 0.0, 1.0);

        // project to raw space
        let position = self.ui_space_matrix * position;

        // project to world space
        let position = self.screen_space_matrix.inverse() * position;

        glam::Vec2::new(position.x(), position.y())
    }

    pub fn world_space_to_ui_space(
        &self,
        world_position: glam::Vec3,
    ) -> glam::Vec2 {
        // input is a position in pixels
        let position = glam::Vec4::new(world_position.x(), world_position.y(), world_position.z(), 1.0);

        // project to raw space
        let position = self.world_space_matrix * position;

        // project to world space
        let position = self.ui_space_matrix.inverse() * position;

        glam::Vec2::new(position.x(), position.y())
    }

    pub fn ui_space_delta_to_world_space_delta(
        &self,
        ui_space_delta: glam::Vec2,
    ) -> glam::Vec3 {
        // Find the world space delta
        let world_space_zero = self.ui_space_to_world_space(glam::Vec2::zero());
        self.ui_space_to_world_space(ui_space_delta) - world_space_zero
    }
}
