#[derive(Copy, Clone)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

impl ViewportSize {
    pub fn new(
        width: u32,
        height: u32,
    ) -> Self {
        ViewportSize { width, height }
    }
}

// this is based on window size (i.e. pixels)
// bottom-left: (0, 0)
// top-right: (window_width_in_pixels, window_height_in_pixels)
fn calculate_ui_space_matrix(viewport_size: ViewportSize) -> glam::Mat4 {
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::from([0.0, 0.0, 5.0]),
        glam::Vec3::from([0.0, 0.0, 0.0]),
        glam::Vec3::from([0.0, 1.0, 0.0]),
    );

    let projection = glam::Mat4::orthographic_rh(
        0.0,
        viewport_size.width as f32,
        0.0,
        viewport_size.height as f32,
        -100.0,
        100.0,
    );

    projection * view
}

// this is a virtual coordinate system
// top-left: (0, 0)
// bottom-right: (600 * aspect_ratio, 600) where aspect_ratio is window_width / window_height
fn calculate_screen_space_matrix(
    _viewport_size: ViewportSize,
    view_half_extents: glam::Vec2,
) -> glam::Mat4 {
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::from([0.0, 0.0, 5.0]),
        glam::Vec3::from([0.0, 0.0, 0.0]),
        glam::Vec3::from([0.0, 1.0, 0.0]),
    );

    let projection = glam::Mat4::orthographic_rh(
        0.0,
        view_half_extents.x() * 2.0,
        view_half_extents.y() * 2.0,
        0.0,
        -100.0,
        100.0,
    );

    projection * view
}

// this is a virtual coordinate system where h = 600 and w = 600 * aspect_ratio where
// aspect_ratio is window_width / window_height
// top-left: (-w/2, -h/2)
// bottom-right: (w/2, h/2)
fn calculate_world_space_matrix(
    _viewport_size: ViewportSize,
    position: glam::Vec3,
    view_half_extents: glam::Vec2,
) -> glam::Mat4 {
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::from([0.0, 0.0, 5.0]),
        glam::Vec3::from([0.0, 0.0, 0.0]),
        glam::Vec3::from([0.0, 1.0, 0.0]),
    );

    let projection = glam::Mat4::orthographic_rh(
        position.x() - view_half_extents.x(),
        position.x() + view_half_extents.x(),
        position.y() + view_half_extents.y(),
        position.y() - view_half_extents.y(),
        -100.0,
        100.0,
    );

    projection * view
}

pub struct ViewportResource {
    view_half_extents: glam::Vec2,
    ui_space_matrix: glam::Mat4,
    screen_space_matrix: glam::Mat4,
    screen_space_dimensions: glam::Vec2,
    world_space_camera_position: glam::Vec3,
    world_space_matrix: glam::Mat4,
}

// UI space: pixels, top-left: (0, 0), bottom-right: (window width in pixels, window height in pixels)
// Raw space: top-left: (-1, -1), bottom-right: (1, 1)
// world space: x positive to the right, y positive going up. width/values depend on camera
// screen space: top-left: (0, 600), bottom-right: (+x, 0) where +x is 600 * screen ratio (i.e. 1066 = ((16/9 * 600) for a 16:9 screen)
impl ViewportResource {
    fn empty() -> Self {
        ViewportResource {
            view_half_extents: glam::Vec2::zero(),
            ui_space_matrix: glam::Mat4::zero(),
            screen_space_matrix: glam::Mat4::zero(),
            screen_space_dimensions: glam::Vec2::zero(),
            world_space_camera_position: glam::Vec3::zero(),
            world_space_matrix: glam::Mat4::zero(),
        }
    }

    pub fn new(
        viewport_size: ViewportSize,
        camera_position: glam::Vec2,
        x_half_extents: f32,
    ) -> Self {
        let mut value = Self::empty();
        value.update(viewport_size, camera_position, x_half_extents);
        value
    }

    pub fn update(
        &mut self,
        viewport_size: ViewportSize,
        camera_position: glam::Vec2,
        x_half_extents: f32,
    ) {
        let y_half_extents =
            x_half_extents / (viewport_size.width as f32 / viewport_size.height as f32);

        self.view_half_extents = glam::Vec2::new(x_half_extents, y_half_extents);

        let camera_position = glam::Vec3::new(camera_position.x(), camera_position.y(), 0.0);
        self.set_ui_space_view(calculate_ui_space_matrix(viewport_size));
        self.set_screen_space_view(
            calculate_screen_space_matrix(viewport_size, self.view_half_extents),
            self.view_half_extents,
        );
        self.set_world_space_view(
            camera_position,
            calculate_world_space_matrix(viewport_size, camera_position, self.view_half_extents),
        );
    }

    pub fn view_half_extents(&self) -> glam::Vec2 {
        self.view_half_extents
    }
    pub fn ui_space_matrix(&self) -> &glam::Mat4 {
        &self.ui_space_matrix
    }
    pub fn screen_space_matrix(&self) -> &glam::Mat4 {
        &self.screen_space_matrix
    }
    pub fn screen_space_dimensions(&self) -> glam::Vec2 {
        self.screen_space_dimensions
    }
    pub fn world_space_camera_position(&self) -> glam::Vec3 {
        self.world_space_camera_position
    }
    pub fn world_space_matrix(&self) -> &glam::Mat4 {
        &self.world_space_matrix
    }

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
        camera_position: glam::Vec3,
        matrix: glam::Mat4,
    ) {
        self.world_space_camera_position = camera_position;
        self.world_space_matrix = matrix;
    }

    pub fn ui_space_to_world_space(
        &self,
        ui_position: glam::Vec2,
    ) -> glam::Vec2 {
        // input is a position in pixels
        let position = glam::Vec4::new(ui_position.x(), ui_position.y(), 0.0, 1.0);

        // project to raw space
        let position = self.ui_space_matrix * position;

        // project to world space
        let position = self.world_space_matrix.inverse() * position;

        glam::Vec2::new(position.x(), position.y())
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
        world_position: glam::Vec2,
    ) -> glam::Vec2 {
        // input is a position in pixels
        let position = glam::Vec4::new(world_position.x(), world_position.y(), 0.0, 1.0);

        // project to raw space
        let position = self.world_space_matrix * position;

        // project to world space
        let position = self.ui_space_matrix.inverse() * position;

        glam::Vec2::new(position.x(), position.y())
    }

    pub fn ui_space_delta_to_world_space_delta(
        &self,
        ui_space_delta: glam::Vec2,
    ) -> glam::Vec2 {
        // Find the world space delta
        let world_space_zero = self.ui_space_to_world_space(glam::Vec2::zero());
        self.ui_space_to_world_space(ui_space_delta) - world_space_zero
    }
}
