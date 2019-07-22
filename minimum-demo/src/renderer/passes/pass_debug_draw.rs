#![cfg_attr(
not(any(feature = "dx12", feature = "metal", feature = "vulkan")),
allow(unused)
)]

use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{render::*, GraphContext, NodeBuffer, NodeImage},
    memory::Dynamic,
    resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle},
    util::types::vertex,
};

use minimum::systems::World;

use nalgebra_glm as glm;

use crate::renderer::{shaders::DEBUG_SHADERS as SHADERS};
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
    fn new(renderer_state: &resources::RenderState) -> UniformArgs {
        UniformArgs {
            mvp: renderer_state.get_world_space_matrix().clone(),
        }
    }
}

const UNIFORM_SIZE: u64 = std::mem::size_of::<UniformArgs>() as u64;

#[derive(Debug, Default)]
pub struct DebugDrawRenderPipelineDesc;

impl<B> SimpleGraphicsPipelineDesc<B, minimum::systems::World> for DebugDrawRenderPipelineDesc
where
    B: gfx_hal::Backend,
{
    type Pipeline = DebugDrawRenderPipeline<B>;

    fn depth_stencil(&self) -> Option<gfx_hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(
        &self,
        factory: &mut Factory<B>,
        _aux: &World,
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
            vertex::PosColor::vertex().gfx_vertex_input_desc(gfx_hal::pso::VertexInputRate::Vertex)
        ];
    }

    fn layout(&self) -> Layout {
        #[cfg(feature = "spirv-reflection")]
        return SHADER_REFLECTION.layout().unwrap();

        #[cfg(not(feature = "spirv-reflection"))]
        return Layout {
            sets: vec![SetLayout {
                bindings: vec![gfx_hal::pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: gfx_hal::pso::DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: gfx_hal::pso::ShaderStageFlags::VERTEX,
                    immutable_samplers: false,
                }],
            }],
            push_constants: vec![],
        };
    }

    /// Returns the InputAssemblerDesc. Defaults to a TriangleList with Restart disabled, can be overriden.
    fn input_assembler(&self) -> gfx_hal::pso::InputAssemblerDesc {
        gfx_hal::pso::InputAssemblerDesc {
            primitive: gfx_hal::Primitive::LineStrip,
            primitive_restart: gfx_hal::pso::PrimitiveRestart::Disabled,
        }
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        aux: &World,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<DebugDrawRenderPipeline<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        log::trace!("DESC BUILD");

        let backbuffer_count = aux.fetch::<resources::RenderState>().backbuffer_count() as usize;
        assert_ne!(backbuffer_count, 0);

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
                factory
                    .device()
                    .write_descriptor_sets(vec![gfx_hal::pso::DescriptorSetWrite {
                        set: descriptor_sets[frame_index].raw(),
                        binding: 0, // Does this binding ID matter?
                        array_offset: 0,
                        descriptors: vec![gfx_hal::pso::Descriptor::Buffer(
                            uniform_buffers[frame_index].raw(),
                            Some(0)..Some(UNIFORM_SIZE),
                        )],
                    }]);
            }
        }

        Ok(DebugDrawRenderPipeline {
            draw_list_vbufs,
            draw_list_ibufs,
            descriptor_sets,
            uniform_buffers,
            draw_list_cmds: vec![],
        })
    }
}

#[derive(Debug)]
struct DrawCommand {
    element_count: u32,
}

impl DrawCommand {
    fn new(element_count: u32) -> Self {
        DrawCommand { element_count }
    }
}

#[derive(Debug)]
pub struct DebugDrawRenderPipeline<B: gfx_hal::Backend> {
    // All of this data is one-per-backbuffer
    uniform_buffers: Vec<Escape<Buffer<B>>>,
    descriptor_sets: Vec<Escape<DescriptorSet<B>>>,
    draw_list_vbufs: Vec<Vec<Escape<Buffer<B>>>>,
    draw_list_ibufs: Vec<Vec<Escape<Buffer<B>>>>,

    // Don't need a separate buffer for this data per-frame, however we do need one
    // list of commands for each vector in the same frame
    draw_list_cmds: Vec<Vec<DrawCommand>>,
}

impl<B> SimpleGraphicsPipeline<B, World> for DebugDrawRenderPipeline<B>
where
    B: gfx_hal::Backend,
{
    type Desc = DebugDrawRenderPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        index: usize,
        aux: &World,
    ) -> PrepareResult {
        log::trace!("prepare");

        //TODO: Reuse these instead of dropping them every frame
        let draw_list_vbufs = &mut self.draw_list_vbufs[index];
        let draw_list_ibufs = &mut self.draw_list_ibufs[index];
        let uniform_buffers = &mut self.uniform_buffers[index];
        let draw_list_cmds = &mut self.draw_list_cmds;

        draw_list_vbufs.clear();
        draw_list_ibufs.clear();
        draw_list_cmds.clear();

        // Used for the view/projection
        //let frame_size = glm::Vec2::new(800.0, 600.0);
        //let window_info = aux.fetch::<crate::gfx::WindowInfo>();
        let renderer_state = aux.fetch::<resources::RenderState>();
        let uniform_args = UniformArgs::new(&renderer_state);

        unsafe {
            factory
                .upload_visible_buffer(uniform_buffers, 0, &[uniform_args])
                .unwrap();
        }

        let mut verts = vec![];
        let mut indices = vec![];
        let mut commands = vec![];

        {
            let mut debug_draw = aux.fetch_mut::<resources::DebugDraw>();
            let line_lists = debug_draw.take_line_lists();

            //TODO: Would be better to pre-size array
            //TODO: This will fail if vertex length is greater than u16::max

            for line_list in line_lists {
                for v in &line_list.points {
                    indices.push(verts.len() as u16);

                    verts.push(vertex::PosColor {
                        position: v.to_homogeneous().xyz().into(),
                        color: line_list.color.into(),
                    });
                }

                commands.push(DrawCommand::new(line_list.points.len() as u32));
            }
        }

        draw_list_cmds.push(commands);

        let vertex_count = verts.len() as u64;
        trace!("vertex count: {:?}", vertex_count);

        if vertex_count > 0 {
            #[cfg(feature = "spirv-reflection")]
            let vbuf_size =
                SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64 * vertex_count;

            #[cfg(not(feature = "spirv-reflection"))]
            let vbuf_size = vertex::PosColor::vertex().stride as u64 * vertex_count;

            let mut vbuf = factory
                .create_buffer(
                    BufferInfo {
                        size: vbuf_size,
                        usage: gfx_hal::buffer::Usage::VERTEX,
                    },
                    Dynamic,
                )
                .unwrap();

            unsafe {
                factory.upload_visible_buffer(&mut vbuf, 0, &verts).unwrap();
            }

            draw_list_vbufs.push(vbuf);

            //INDEX BUFFER
            let ibuf_size = indices.len() as u64 * std::mem::size_of::<u16>() as u64;
            let mut ibuf = factory
                .create_buffer(
                    BufferInfo {
                        size: ibuf_size,
                        usage: gfx_hal::buffer::Usage::INDEX,
                    },
                    Dynamic,
                )
                .unwrap();

            unsafe {
                factory
                    .upload_visible_buffer(&mut ibuf, 0, &indices)
                    .unwrap();
            }

            draw_list_ibufs.push(ibuf);
        }

        return PrepareResult::DrawRecord;
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _aux: &World,
    ) {
        log::trace!("draw");

        let draw_list_vbufs = &self.draw_list_vbufs[index];
        let draw_list_ibufs = &self.draw_list_ibufs[index];

        assert_eq!(draw_list_vbufs.len(), draw_list_ibufs.len());

        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.descriptor_sets[index].raw()),
                std::iter::empty::<u32>(),
            );

            for draw_list_index in 0..draw_list_vbufs.len() {
                encoder.bind_vertex_buffers(0, Some((draw_list_vbufs[draw_list_index].raw(), 0)));

                encoder.bind_index_buffer(
                    draw_list_ibufs[draw_list_index].raw(),
                    0,
                    gfx_hal::IndexType::U16,
                );

                let mut element_begin_index = 0;
                for cmd in &self.draw_list_cmds[draw_list_index] {
                    let element_end_index = element_begin_index + cmd.element_count;
                    encoder.draw_indexed(element_begin_index..element_end_index, 0, 0..1);

                    element_begin_index = element_end_index;
                }
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &World) {}
}
