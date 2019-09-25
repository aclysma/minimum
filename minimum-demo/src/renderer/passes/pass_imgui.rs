#![cfg_attr(
not(any(feature = "dx12", feature = "metal", feature = "vulkan")),
allow(unused)
)]

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::{Factory, ImageState},
    graph::{render::*, GraphContext, NodeBuffer, NodeImage},
    memory::Dynamic,
    resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle},
    texture::Texture,
    util::types::vertex,
    wsi::winit,
};

use crate::base::resource::ResourceMap;

use nalgebra_glm as glm;

use crate::renderer::{shaders::IMGUI_SHADERS as SHADERS, vertex_types::PosTexColor};
use crate::resources;

#[cfg(feature = "spirv-reflection")]
use rendy::shader::SpirvReflection;

#[cfg(not(feature = "spirv-reflection"))]
use vertex::AsVertex;

// This defines a view-projection matrix
#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct UniformArgs {
    pub mvp: glm::Mat4,
}

impl UniformArgs {
    fn new(renderer_state: &crate::resources::RenderState) -> UniformArgs {
        UniformArgs {
            mvp: renderer_state.ui_space_matrix().clone(),
        }
    }
}

const UNIFORM_SIZE: u64 = std::mem::size_of::<UniformArgs>() as u64;

#[derive(Debug, Default)]
pub struct ImguiRenderPipelineDesc;

impl<B> SimpleGraphicsPipelineDesc<B, crate::base::resource::ResourceMap> for ImguiRenderPipelineDesc
where
    B: gfx_hal::Backend,
{
    type Pipeline = ImguiRenderPipeline<B>;

    fn depth_stencil(&self) -> Option<gfx_hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(
        &self,
        factory: &mut Factory<B>,
        _aux: &ResourceMap,
    ) -> rendy::shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>,
        gfx_hal::pso::ElemStride,
        gfx_hal::pso::VertexInputRate,
    )> {
        #[cfg(feature = "spirv-reflection")]
        return vec![SHADER_REFLECTION
            .attributes_range(..)
            .unwrap()
            .gfx_vertex_input_desc(0)];

        #[cfg(not(feature = "spirv-reflection"))]
        return vec![
            PosTexColor::vertex().gfx_vertex_input_desc(gfx_hal::pso::VertexInputRate::Vertex)
        ];
    }

    fn layout(&self) -> Layout {
        #[cfg(feature = "spirv-reflection")]
        return SHADER_REFLECTION.layout().unwrap();

        #[cfg(not(feature = "spirv-reflection"))]
        return Layout {
            sets: vec![SetLayout {
                bindings: vec![
                    gfx_hal::pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: gfx_hal::pso::DescriptorType::SampledImage,
                        count: 1,
                        stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    gfx_hal::pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: gfx_hal::pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: gfx_hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    gfx_hal::pso::DescriptorSetLayoutBinding {
                        binding: 2, // does this binding ID matter? I think it does because binding 0 here steps on the sample image above
                        ty: gfx_hal::pso::DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: gfx_hal::pso::ShaderStageFlags::VERTEX,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: vec![],
        };
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &ResourceMap,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<ImguiRenderPipeline<B>, gfx_hal::pso::CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        log::trace!("DESC BUILD");

        let backbuffer_count = aux.fetch::<resources::RenderState>().backbuffer_count() as usize;
        assert_ne!(backbuffer_count, 0);

        let texture = {
            let mut imgui_manager = aux.fetch_mut::<resources::ImguiManager>();
            let font_atlas = imgui_manager.font_atlas_texture();

            let kind = gfx_hal::image::Kind::D2(font_atlas.width, font_atlas.height, 1, 1);
            let view_kind = gfx_hal::image::ViewKind::D2;
            let sampler_info = gfx_hal::image::SamplerInfo::new(
                gfx_hal::image::Filter::Linear,
                gfx_hal::image::WrapMode::Clamp,
            );

            let texture_builder = rendy::texture::TextureBuilder::new()
                .with_raw_data(font_atlas.data, gfx_hal::format::Format::Rgba8Srgb)
                //no swizzle
                .with_data_width(font_atlas.width)
                .with_data_height(font_atlas.height)
                .with_kind(kind)
                .with_view_kind(view_kind)
                .with_sampler_info(sampler_info);

            texture_builder
                .build(
                    ImageState {
                        queue,
                        stage: gfx_hal::pso::PipelineStage::FRAGMENT_SHADER,
                        access: gfx_hal::image::Access::SHADER_READ,
                        layout: gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                    },
                    factory,
                )
                .unwrap()
        };

        let mut uniform_buffers = vec![];
        let mut descriptor_sets = vec![];
        let mut draw_list_vbufs = vec![];
        let mut draw_list_ibufs = vec![];

        // Create uniform buffers and descriptor sets for each backbuffer, since this data can
        // change frame-to-frame
        for _frame_index in 0..backbuffer_count {
            let uniform_buffer = factory
                .create_buffer(
                    BufferInfo {
                        size: UNIFORM_SIZE,
                        usage: gfx_hal::buffer::Usage::UNIFORM,
                    },
                    Dynamic,
                )
                .unwrap();

            let descriptor_set = factory
                .create_descriptor_set(set_layouts[0].clone())
                .unwrap();

            uniform_buffers.push(uniform_buffer);
            descriptor_sets.push(descriptor_set);
            draw_list_vbufs.push(vec![]);
            draw_list_ibufs.push(vec![]);
        }

        // Write descriptor sets for each backbuffer
        unsafe {
            for frame_index in 0..backbuffer_count {
                use gfx_hal::device::Device;

                let dsw = vec![
                    gfx_hal::pso::DescriptorSetWrite {
                        set: descriptor_sets[frame_index].raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: vec![gfx_hal::pso::Descriptor::Image(
                            texture.view().raw(),
                            gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                        )],
                    },
                    gfx_hal::pso::DescriptorSetWrite {
                        set: descriptor_sets[frame_index].raw(),
                        binding: 1,
                        array_offset: 0,
                        descriptors: vec![gfx_hal::pso::Descriptor::Sampler(
                            texture.sampler().raw(),
                        )],
                    },
                    gfx_hal::pso::DescriptorSetWrite {
                        set: descriptor_sets[frame_index].raw(),
                        binding: 0, // Does this binding ID matter?
                        array_offset: 0,
                        descriptors: vec![gfx_hal::pso::Descriptor::Buffer(
                            uniform_buffers[frame_index].raw(),
                            Some(0)..Some(UNIFORM_SIZE),
                        )],
                    },
                ];
                factory.device().write_descriptor_sets(dsw);
            }
        }

        let quad_vertex_data = [
            PosTexColor {
                position: [-0.5, 0.33].into(),
                tex_coord: [0.0, 1.0].into(),
                color: [255, 255, 255, 255].into(),
            },
            PosTexColor {
                position: [0.5, 0.33].into(),
                tex_coord: [1.0, 1.0].into(),
                color: [255, 255, 255, 255].into(),
            },
            PosTexColor {
                position: [0.5, -0.33].into(),
                tex_coord: [1.0, 0.0].into(),
                color: [255, 255, 255, 255].into(),
            },
            PosTexColor {
                position: [-0.5, 0.33].into(),
                tex_coord: [0.0, 1.0].into(),
                color: [255, 255, 255, 255].into(),
            },
            PosTexColor {
                position: [0.5, -0.33].into(),
                tex_coord: [1.0, 0.0].into(),
                color: [255, 255, 255, 255].into(),
            },
            PosTexColor {
                position: [-0.5, -0.33].into(),
                tex_coord: [0.0, 0.0].into(),
                color: [255, 255, 255, 255].into(),
            },
        ];

        #[cfg(feature = "spirv-reflection")]
        let vbuf_size = SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64
            * quad_vertex_data.len() as u64;

        #[cfg(not(feature = "spirv-reflection"))]
        let vbuf_size = PosTexColor::vertex().stride as u64 * quad_vertex_data.len() as u64;

        let mut quad_vbuf = factory
            .create_buffer(
                BufferInfo {
                    size: vbuf_size,
                    usage: gfx_hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();

        unsafe {
            // Fresh buffer.
            factory
                .upload_visible_buffer(&mut quad_vbuf, 0, &quad_vertex_data)
                .unwrap();
        }

        Ok(ImguiRenderPipeline {
            texture,
            quad_vbuf,
            draw_list_vbufs,
            draw_list_ibufs,
            descriptor_sets,
            uniform_buffers,
        })
    }
}

#[derive(Debug)]
pub struct ImguiRenderPipeline<B: gfx_hal::Backend> {
    texture: Texture<B>,
    quad_vbuf: Escape<Buffer<B>>,
    uniform_buffers: Vec<Escape<Buffer<B>>>,
    descriptor_sets: Vec<Escape<DescriptorSet<B>>>,
    draw_list_vbufs: Vec<Vec<Escape<Buffer<B>>>>,
    draw_list_ibufs: Vec<Vec<Escape<Buffer<B>>>>,
}

impl<B> SimpleGraphicsPipeline<B, ResourceMap> for ImguiRenderPipeline<B>
where
    B: gfx_hal::Backend,
{
    type Desc = ImguiRenderPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        index: usize,
        aux: &ResourceMap,
    ) -> PrepareResult {
        log::trace!("prepare");

        //TODO: Reuse these instead of dropping them every frame
        let draw_list_vbufs = &mut self.draw_list_vbufs[index];
        let draw_list_ibufs = &mut self.draw_list_ibufs[index];
        let uniform_buffers = &mut self.uniform_buffers[index];

        draw_list_vbufs.clear();
        draw_list_ibufs.clear();

        let render_state = aux.fetch::<crate::resources::RenderState>();

        unsafe {
            let mut imgui_manager = aux.fetch_mut::<resources::ImguiManager>();
            let window = aux.fetch::<winit::window::Window>();
            if imgui_manager.is_frame_started() {
                imgui_manager.render(&window);

                let uniform_args = UniformArgs::new(&render_state);

                factory
                    .upload_visible_buffer(uniform_buffers, 0, &[uniform_args])
                    .unwrap();

                let draw_data = imgui_manager.draw_data();
                if draw_data.is_none() {
                    warn!("get_draw_data returned None");
                    return PrepareResult::DrawRecord;
                }

                let draw_lists = draw_data.unwrap().draw_lists();

                for draw_list in draw_lists {
                    // VERTEX BUFFER
                    let vertex_count = draw_list.vtx_buffer().len() as u64;

                    #[cfg(feature = "spirv-reflection")]
                    let vbuf_size = SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64
                        * vertex_count;

                    #[cfg(not(feature = "spirv-reflection"))]
                    let vbuf_size = PosTexColor::vertex().stride as u64 * vertex_count;

                    let mut vbuf = factory
                        .create_buffer(
                            BufferInfo {
                                size: vbuf_size,
                                usage: gfx_hal::buffer::Usage::VERTEX,
                            },
                            Dynamic,
                        )
                        .unwrap();

                    factory
                        .upload_visible_buffer(&mut vbuf, 0, &draw_list.vtx_buffer())
                        .unwrap();

                    draw_list_vbufs.push(vbuf);

                    //INDEX BUFFER
                    let ibuf_size =
                        draw_list.idx_buffer().len() as u64 * std::mem::size_of::<u16>() as u64;
                    let mut ibuf = factory
                        .create_buffer(
                            BufferInfo {
                                size: ibuf_size,
                                usage: gfx_hal::buffer::Usage::INDEX,
                            },
                            Dynamic,
                        )
                        .unwrap();

                    factory
                        .upload_visible_buffer(&mut ibuf, 0, &draw_list.idx_buffer())
                        .unwrap();

                    draw_list_ibufs.push(ibuf);
                }
            }
        };

        return PrepareResult::DrawRecord;
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        aux: &ResourceMap,
    ) {
        log::trace!("draw");

        let draw_list_vbufs = &self.draw_list_vbufs[index];
        let draw_list_ibufs = &self.draw_list_ibufs[index];

        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.descriptor_sets[index].raw()),
                std::iter::empty::<u32>(),
            );

            encoder.bind_vertex_buffers(0, Some((self.quad_vbuf.raw(), 0)));
            encoder.draw(0..6, 0..1);
        }

        unsafe {
            let mut imgui_context = aux.fetch_mut::<resources::ImguiManager>();
            let draw_data = imgui_context.draw_data();

            if let Some(draw_data) = draw_data {
                //TODO: Verify the draw list index doesn't exceed the vbuf/ibuf list
                let mut draw_list_index = 0;
                for draw_list in draw_data.draw_lists() {
                    // If for some reason we didn't actually create the buffers, don't try to draw them
                    if draw_list_index >= draw_list_vbufs.len()
                        || draw_list_index >= draw_list_ibufs.len()
                    {
                        break;
                    }

                    encoder
                        .bind_vertex_buffers(0, Some((draw_list_vbufs[draw_list_index].raw(), 0)));

                    encoder.bind_index_buffer(
                        draw_list_ibufs[draw_list_index].raw(),
                        0,
                        gfx_hal::IndexType::U16,
                    );

                    let mut element_begin_index: u32 = 0;
                    for cmd in draw_list.commands() {
                        match cmd {
                            imgui::DrawCmd::Elements {
                                count,
                                cmd_params: imgui::DrawCmdParams {
                                    clip_rect,
                                    //texture_id,
                                    ..
                                }
                            } => {
                                let element_end_index = element_begin_index + count as u32;

                                encoder.set_scissors(0, &[
                                    gfx_hal::pso::Rect {
                                        x: ((clip_rect[0] - draw_data.display_pos[0]) * draw_data.framebuffer_scale[0]) as i16,
                                        y: ((clip_rect[1] - draw_data.display_pos[1]) * draw_data.framebuffer_scale[1]) as i16,
                                        w: ((clip_rect[2] - clip_rect[0] - draw_data.display_pos[0]) * draw_data.framebuffer_scale[0]) as i16,
                                        h: ((clip_rect[3] - clip_rect[1] - draw_data.display_pos[1]) * draw_data.framebuffer_scale[1]) as i16,
                                    }
                                ]);

                                encoder.draw_indexed(
                                    element_begin_index..element_end_index,
                                    0,
                                    0..1,
                                );

                                element_begin_index = element_end_index;
                            }
                            _ => panic!("unexpected draw command"),
                        }
                    }

                    draw_list_index += 1;
                }
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &ResourceMap) {}
}
