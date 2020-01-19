pub struct CameraState {
    ui_space_matrix: glm::Mat4,
    screen_space_matrix: glm::Mat4,
    screen_space_dimensions: glm::Vec2,
    world_space_camera_position: glm::Vec3,
    world_space_matrix: glm::Mat4,
}

// UI space: pixels, top-left: (0, 0), bottom-right: (window width in pixels, window height in pixels)
// Raw space: top-left: (-1, -1), bottom-right: (1, 1)
// world space: x positive to the right, y positive going up. width/values depend on camera
// screen space: top-left: (0, 600), bottom-right: (+x, 0) where +x is 600 * screen ratio (i.e. 1066 = ((16/9 * 600) for a 16:9 screen)
impl CameraState {
    //TODO: Find some alternative that prevents this from having to ever be in an invalid state
    pub fn empty() -> Self {
        CameraState {
            ui_space_matrix: glm::zero(),
            screen_space_matrix: glm::zero(),
            screen_space_dimensions: glm::zero(),
            world_space_camera_position: glm::zero(),
            world_space_matrix: glm::zero(),
        }
    }

    pub fn init(
        &mut self,
        ui_space_matrix: glm::Mat4,
        screen_space_matrix: glm::Mat4,
        screen_space_dimensions: glm::Vec2,
        world_space_camera_position: glm::Vec3,
        world_space_matrix: glm::Mat4,
    ) {
        self.ui_space_matrix = ui_space_matrix;
        self.screen_space_matrix = screen_space_matrix;
        self.screen_space_dimensions = screen_space_dimensions;
        self.world_space_camera_position = world_space_camera_position;
        self.world_space_matrix = world_space_matrix;
    }

    pub fn ui_space_matrix(&self) -> &glm::Mat4 {
        &self.ui_space_matrix
    }
    pub fn screen_space_matrix(&self) -> &glm::Mat4 {
        &self.screen_space_matrix
    }
    pub fn screen_space_dimensions(&self) -> glm::Vec2 {
        self.screen_space_dimensions
    }
    pub fn world_space_camera_position(&self) -> glm::Vec3 {
        self.world_space_camera_position
    }
    pub fn world_space_matrix(&self) -> &glm::Mat4 {
        &self.world_space_matrix
    }

    pub fn set_ui_space_view(&mut self, matrix: glm::Mat4) {
        self.ui_space_matrix = matrix;
    }

    pub fn set_screen_space_view(&mut self, matrix: glm::Mat4, dimensions: glm::Vec2) {
        self.screen_space_matrix = matrix;
        self.screen_space_dimensions = dimensions;
    }

    pub fn set_world_space_view(&mut self, camera_position: glm::Vec3, matrix: glm::Mat4) {
        self.world_space_camera_position = camera_position;
        self.world_space_matrix = matrix;
    }

    pub fn ui_space_to_world_space(&self, ui_position: glm::Vec2) -> glm::Vec2 {
        // input is a position in pixels
        let position = glm::vec4(ui_position.x, ui_position.y, 0.0, 1.0);

        // project to raw space
        let position = self.ui_space_matrix * position;

        // project to world space
        let position = glm::inverse(&self.world_space_matrix) * position;

        position.xy()
    }

    pub fn ui_space_to_screen_space(&self, ui_position: glm::Vec2) -> glm::Vec2 {
        // input is a position in pixels
        let position = glm::vec4(ui_position.x, ui_position.y, 0.0, 1.0);

        // project to raw space
        let position = self.ui_space_matrix * position;

        // project to world space
        let position = glm::inverse(&self.screen_space_matrix) * position;

        position.xy()
    }

    pub fn world_space_to_ui_space(&self, world_position: glm::Vec2) -> glm::Vec2 {
        // input is a position in pixels
        let position = glm::vec4(world_position.x, world_position.y, 0.0, 1.0);

        // project to raw space
        let position = self.world_space_matrix * position;

        // project to world space
        let position = glm::inverse(&self.ui_space_matrix) * position;

        position.xy()
    }


    pub fn ui_space_delta_to_world_space_delta(&self, ui_space_delta: glm::Vec2) -> glm::Vec2 {
        // Find the world space delta
        let world_space_zero = self.ui_space_to_world_space(glm::zero());
        self.ui_space_to_world_space(ui_space_delta) - world_space_zero
    }
}
