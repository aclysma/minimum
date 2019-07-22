

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

    pub fn backbuffer_count(&self) -> u32 { self.backbuffer_count }

    pub fn get_ui_space_matrix(&self) -> &glm::Mat4 {
        &self.ui_space_matrix
    }

    pub fn set_ui_space_view(&mut self, matrix: glm::Mat4) {
        self.ui_space_matrix = matrix;
    }

    pub fn get_screen_space_matrix(&self) -> &glm::Mat4 {
        &self.screen_space_matrix
    }

    pub fn get_screen_space_dimensions(&self) -> glm::Vec2 {
        self.screen_space_dimensions
    }

    pub fn set_screen_space_view(&mut self, matrix: glm::Mat4, dimensions: glm::Vec2) {
        self.screen_space_matrix = matrix;
        self.screen_space_dimensions = dimensions;
    }

    pub fn get_world_space_camera_position(&self) -> glm::Vec3 {
        self.world_space_camera_position
    }

    pub fn get_world_space_matrix(&self) -> &glm::Mat4 {
        &self.world_space_matrix
    }

    pub fn set_world_space_view(&mut self, camera_position: glm::Vec3, matrix: glm::Mat4) {
        self.world_space_camera_position = camera_position;
        self.world_space_matrix = matrix;
    }
}
