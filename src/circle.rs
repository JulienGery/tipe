use crate::types::{self, MyVertex, Putain};
use std::sync::Arc;

use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::persistent::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{
    allocator::CommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModule;

#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct Circle {
    #[format(R32G32B32_SFLOAT)]
    pub circle_position: [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
    #[format(R32_SFLOAT)]
    pub radius: f32,
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 local_position;

            //per-instance data
            layout(location = 1) in vec3 circle_position;
            layout(location = 2) in vec4 color;
            layout(location = 3) in float radius;

            //uniform data
            layout(set = 0, binding = 0) uniform UBO
            {
                mat4 projection;
                mat4 modelview;
            } ubo;


            //out
            layout(location = 0) out vec3 o_position;
            layout(location = 1) out vec4 o_color;

            void main() {
                o_position = local_position;
                o_color = color;

                gl_Position = ubo.projection * ubo.modelview * vec4(local_position * radius + circle_position, 1.0);
            }
            ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec4 color;

            layout(location = 0) out vec4 f_color;

            void main() {
                // float distance = length(position);
                // float jsp = step(distance, 1.);
                // if (jsp != 1.)
                //     discard;
                //
                // f_color = vec4(1., 1., 1., 1.) * jsp;
                f_color = color;
            }
        ",
    }
}

impl Circle {
    pub fn vertex() -> Vec<MyVertex> {
        let top_right = MyVertex {
            local_position: [1., 1., 0.],
        };
        let top_left = MyVertex {
            local_position: [-1., 1., 0.],
        };
        let bottom_left = MyVertex {
            local_position: [-1., -1., 0.],
        };
        let bottom_right = MyVertex {
            local_position: [1., -1., 0.],
        };

        vec![
            top_right,
            top_left.clone(),
            bottom_right.clone(),
            bottom_left,
            top_left.clone(),
            bottom_right.clone(),
        ]
    }

    pub fn new(radius: f32, position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            radius,
            color,
            circle_position: position,
        }
    }
}

// #[derive(Debug)]
pub struct CircleManadger {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    circles: Vec<Circle>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    vertex_buffer: Option<Subbuffer<[MyVertex]>>,
    instance_buffer: Option<Subbuffer<[Circle]>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    descriptor_set : Option<Arc<PersistentDescriptorSet>>,
    // uniform_buffer : Option<SubbufferAllocator>
}

impl CircleManadger {
    pub fn new(device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        Self {
            device,
            memory_allocator,
            circles: vec![],
            vs,
            fs,
            pipeline: None,
            vertex_buffer: None,
            instance_buffer: None,
            descriptor_set : None
            // uniform_buffer : None
        }
    }

    pub fn clear_buffer(&mut self) {
        self.vertex_buffer = None;
        self.instance_buffer = None;
    }

    pub fn clear(&mut self) {
        self.clear_buffer();
        self.circles = vec![];
    }

    pub fn append(&mut self, other: &mut Vec<Circle>) {
        self.circles.append(other);
    }

    pub fn build_pipeline(
        &mut self,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
        descriptor_set_allocator : &StandardDescriptorSetAllocator,
        buffer: Subbuffer<impl ?Sized>
    ) -> &mut Self {
        let vs = self.vs.entry_point("main").unwrap();
        let fs = self.fs.entry_point("main").unwrap();

        let vertex_input_state = [MyVertex::per_vertex(), Circle::per_instance()]
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let pipeline = GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                        )),
                        subpass: Some(subpass.into()),
                        ..GraphicsPipelineCreateInfo::layout(layout)
            },
            ).unwrap();

        // let uniform_data = types::MyUniformBuffer::new().data();

        // let data_buffer = {
        //     let subbuffer = self.uniform_buffer.take().unwrap().allocate_sized().unwrap();
        //     *subbuffer.write().unwrap() = uniform_data;
        //
        //     subbuffer
        // };


        //should be moved to main class
        let pipeline_layout = pipeline.layout();
        let descriptor_set_layouts = pipeline_layout.set_layouts();
        //
        let descriptor_set_layout_index = 0;
        let descriptor_set_layout = descriptor_set_layouts
            .get(descriptor_set_layout_index)
            .unwrap();

        let descriptor_set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, buffer)], // 0 is the binding
            [],
            )
            .unwrap();

        self.descriptor_set = Some(descriptor_set);
        self.pipeline = Some(pipeline);
        self
    }

    pub fn bake(&mut self) -> &mut Self {
        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            Circle::vertex(),
        )
        .unwrap();

        let instance_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.circles.clone(),
        )
        .unwrap();

        // let uniform_buffer = SubbufferAllocator::new(
        //     self.memory_allocator.clone(),
        //     SubbufferAllocatorCreateInfo {
        //         buffer_usage: BufferUsage::UNIFORM_BUFFER,
        //         memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
        //             | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
        //             ..Default::default()
        //     },
        //     );

        self.vertex_buffer = Some(vertex_buffer);
        self.instance_buffer = Some(instance_buffer);
        self
    }

    pub fn draw<A>(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<A>, A>,
    ) where
        A: CommandBufferAllocator,
    {
        builder
            .bind_pipeline_graphics(self.pipeline.clone().unwrap())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.clone().unwrap().layout().clone(),
                0,
                self.descriptor_set.clone().unwrap())
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone().unwrap())
            .unwrap()
            .bind_vertex_buffers(1, self.instance_buffer.clone().unwrap())
            .unwrap()
            .draw(
                self.vertex_buffer.clone().unwrap().len() as u32,
                self.instance_buffer.clone().unwrap().len() as u32,
                0,
                0,
            )
            .unwrap();
    }
}
