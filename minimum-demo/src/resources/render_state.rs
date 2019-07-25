pub struct RenderState {
    backbuffer_count: u32,
    ui_space_matrix: glm::Mat4,
    screen_space_matrix: glm::Mat4,
    screen_space_dimensions: glm::Vec2,
    world_space_camera_position: glm::Vec3,
    world_space_matrix: glm::Mat4,
}

impl RenderState {
    //TODO: Find some alternative that prevents this from having to ever be in an invalid state
    pub fn empty() -> Self {
        RenderState {
            backbuffer_count: 0,
            ui_space_matrix: glm::zero(),
            screen_space_matrix: glm::zero(),
            screen_space_dimensions: glm::zero(),
            world_space_camera_position: glm::zero(),
            world_space_matrix: glm::zero(),
        }
    }

    pub fn init(
        &mut self,
        backbuffer_count: u32,
        ui_space_matrix: glm::Mat4,
        screen_space_matrix: glm::Mat4,
        screen_space_dimensions: glm::Vec2,
        world_space_camera_position: glm::Vec3,
        world_space_matrix: glm::Mat4,
    ) {
        self.backbuffer_count = backbuffer_count;
        self.ui_space_matrix = ui_space_matrix;
        self.screen_space_matrix = screen_space_matrix;
        self.screen_space_dimensions = screen_space_dimensions;
        self.world_space_camera_position = world_space_camera_position;
        self.world_space_matrix = world_space_matrix;
    }

    pub fn backbuffer_count(&self) -> u32 {
        self.backbuffer_count
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

    // this is based on window size (i.e. pixels)
    // bottom-left: (0, 0)
    // top-right: (window_width_in_pixels, window_height_in_pixels)
    pub fn set_ui_space_view(&mut self, matrix: glm::Mat4) {
        self.ui_space_matrix = matrix;
    }

    // this is a virtual coordinate system
    // top-left: (0, 0)
    // bottom-right: (600 * aspect_ratio, 600) where aspect_ratio is window_width / window_height
    pub fn set_screen_space_view(&mut self, matrix: glm::Mat4, dimensions: glm::Vec2) {
        self.screen_space_matrix = matrix;
        self.screen_space_dimensions = dimensions;
    }

    // this is a virtual coordinate system where h = 600 and w = 600 * aspect_ratio where
    // aspect_ratio is window_width / window_height
    // top-left: (-w/2, -h/2)
    // bottom-right: (w/2, h/2)
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
}
