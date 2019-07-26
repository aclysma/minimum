#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
type Backend = rendy::vulkan::Backend;

mod passes;
mod shaders;
mod vertex_types;

use rendy::{
    command::Families, factory::Config, factory::Factory, graph::present::PresentNode,
    graph::render::RenderGroupBuilder, graph::render::SimpleGraphicsPipeline, graph::Graph,
    graph::GraphBuilder,
};

use minimum::systems::World;

use crate::renderer::passes::{DebugDrawRenderPipeline, ImguiRenderPipeline};

use crate::resources;

pub struct Renderer {
    factory: Factory<Backend>,
    families: Families<Backend>,
    graph: Option<Graph<Backend, World>>,
    camera_position: glm::Vec3,
    camera_zoom: f32,
}

impl Renderer {
    pub fn new() -> Self {
        // Use default rendy configuration.. this allows to inject device, heap, and queue
        // selection
        let config: Config = Default::default();

        // The factory is high-level owner of the device, instance, and manges memory, resources and
        // queue families.
        // Families represents the queue families on the device
        let (factory, families): (Factory<Backend>, _) = rendy::factory::init(config).unwrap();

        Renderer {
            factory,
            families,
            graph: None,
            camera_position: glm::zero(),
            camera_zoom: 1.0,
        }
    }

    pub fn camera_position(&self) -> glm::Vec3 {
        self.camera_position
    }

    pub fn camera_zoom(&self) -> f32 {
        self.camera_zoom
    }

    pub fn init_render_graph(
        &mut self,
        window: &winit::window::Window,
        world: &minimum::systems::World,
    ) {
        let surface = self.factory.create_surface(window);

        // GraphBuilder gives us a declarative interface for describing what/how to render. Using this
        // structure rather than directly making calls on a GPU backend means much of the error
        // handling and recovery (such as the device being lost) are automatically handled
        let mut graph_builder = GraphBuilder::<Backend, World>::new();

        // The frame starts with a cleared color buffer
        let color = graph_builder.create_image(
            gfx_hal::Surface::kind(surface.raw()),
            1,
            self.factory.get_surface_format(&surface),
            Some(gfx_hal::command::ClearValue::Color(
                [0.1, 0.1, 0.1, 1.0].into(),
            )),
        );

        //        let depth = graph_builder.create_image(
        //            gfx_hal::Surface::kind(surface.raw()),
        //            1,
        //            gfx_hal::format::Format::D16Unorm,
        //            Some(gfx_hal::command::ClearValue::DepthStencil(
        //                gfx_hal::command::ClearDepthStencil(1.0, 0),
        //            )),
        //        );

        // Render imgui
        let pass0 = graph_builder.add_node(
            DebugDrawRenderPipeline::builder()
                .into_subpass()
                .with_color(color)
                .into_pass(),
        );

        let pass1 = graph_builder.add_node(
            ImguiRenderPipeline::builder()
                .with_dependency(pass0)
                .into_subpass()
                .with_color(color)
                .into_pass(),
        );

        let present_builder =
            PresentNode::builder(&self.factory, surface, color).with_dependency(pass1);

        let swapchain_backbuffer_count = present_builder.image_count();
        world.fetch_mut::<resources::RenderState>().init(
            swapchain_backbuffer_count,
            Renderer::calculate_ui_space_matrix(window),
            Renderer::calculate_screen_space_matrix(window),
            Renderer::calculate_screen_space_dimensions(window),
            glm::zero(),
            Renderer::calculate_world_space_matrix(window, glm::zero(), 1.0),
        );

        // Then present the pass to the screen
        graph_builder.add_node(present_builder);

        self.graph = Some(
            graph_builder
                .build(&mut self.factory, &mut self.families, world)
                .unwrap(),
        );
    }

    //pub fn update(&mut self) {
    //        self.factory.maintain(&mut self.families);
    //    }

    pub fn render(&mut self, window: &winit::window::Window, world: &minimum::systems::World) {
        self.factory.maintain(&mut self.families);

        // Update the render state
        {
            // Here you can recalculate where you want the camera to be.
            self.camera_position = glm::Vec3::new(0.0, 0.0, 5.0);

            // Zoom in/out
            self.camera_zoom = 1.0;

            let mut renderer_state = world.fetch_mut::<resources::RenderState>();
            renderer_state.set_ui_space_view(Renderer::calculate_ui_space_matrix(window));
            renderer_state.set_screen_space_view(
                Renderer::calculate_screen_space_matrix(window),
                Renderer::calculate_screen_space_dimensions(window),
            );
            renderer_state.set_world_space_view(
                self.camera_position,
                Renderer::calculate_world_space_matrix(
                    window,
                    self.camera_position,
                    self.camera_zoom,
                ),
            );
        }

        // Kick off rendering
        match &mut self.graph {
            Some(x) => x.run(&mut self.factory, &mut self.families, world),
            _ => {}
        }
    }

    pub fn dispose(mut self, world: &minimum::systems::World) {
        match self.graph {
            Some(x) => x.dispose(&mut self.factory, world),
            _ => {}
        }
    }

    // this is based on window size (i.e. pixels)
    // bottom-left: (0, 0)
    // top-right: (window_width_in_pixels, window_height_in_pixels)
    fn calculate_ui_space_matrix(window: &winit::window::Window) -> glm::Mat4 {
        let logical_size = window.inner_size();

        let view = glm::look_at_rh(
            &glm::make_vec3(&[0.0, 0.0, 5.0]),
            &glm::make_vec3(&[0.0, 0.0, 0.0]),
            &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
        );

        let projection = glm::ortho_rh_zo(
            0.0,
            logical_size.width as f32,
            0.0,
            logical_size.height as f32,
            -100.0,
            100.0,
        );

        projection * view
    }

    fn calculate_screen_space_dimensions(window: &winit::window::Window) -> glm::Vec2 {
        let logical_size = window.inner_size();
        let ratio = (logical_size.width / logical_size.height) as f32;

        glm::Vec2::new(600.0 * ratio, 600.0)
    }

    // this is a virtual coordinate system
    // top-left: (0, 0)
    // bottom-right: (600 * aspect_ratio, 600) where aspect_ratio is window_width / window_height
    fn calculate_screen_space_matrix(window: &winit::window::Window) -> glm::Mat4 {
        let screen_space_dimensions = Renderer::calculate_screen_space_dimensions(window);

        let view_extent = glm::Vec2::new(
            screen_space_dimensions.x / 2.0,
            screen_space_dimensions.y / 2.0,
        );

        let view = glm::look_at_rh(
            &glm::make_vec3(&[0.0, 0.0, 5.0]),
            &glm::make_vec3(&[0.0, 0.0, 0.0]),
            &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
        );

        let projection = glm::ortho_rh_zo(
            0.0,
            view_extent.x * 2.0,
            view_extent.y * 2.0,
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
        window: &winit::window::Window,
        position: glm::Vec3,
        zoom: f32,
    ) -> glm::Mat4 {
        let screen_space_dimensions = Renderer::calculate_screen_space_dimensions(window);

        let mut view_extent = glm::Vec2::new(
            screen_space_dimensions.x / 2.0,
            screen_space_dimensions.y / 2.0,
        );

        view_extent *= 1.0 / zoom;

        let view = glm::look_at_rh(
            &glm::make_vec3(&[0.0, 0.0, 5.0]),
            &glm::make_vec3(&[0.0, 0.0, 0.0]),
            &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
        );

        let projection = glm::ortho_rh_zo(
            position.x - view_extent.x,
            position.x + view_extent.x,
            position.y + view_extent.y,
            position.y - view_extent.y,
            -100.0,
            100.0,
        );

        projection * view
    }
}