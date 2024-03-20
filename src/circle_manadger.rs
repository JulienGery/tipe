use std::{collections::HashMap, sync::Arc, usize};

use glam::Mat4;
use vulkano::{buffer::{allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}, Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::CommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, descriptor_set::{allocator::{StandardDescriptorSetAlloc, StandardDescriptorSetAllocator}, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{RenderPass, Subpass}, shader::ShaderModule};
use vulkano::pipeline::{Pipeline};
use winit::window::WindowId;

use crate::{circles::{self, vs, Circle, MyVertex}, plot::Plot};

pub struct CircleManadger {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    vertex_buffer: Option<Subbuffer<[MyVertex]>>,
    instance_size: HashMap<WindowId, usize>,
    instance_offset: HashMap<WindowId, usize>,
    instance_buffer: Option<Subbuffer<[Circle]>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    // descriptor_set : HashMap<WindowId, Vec<Arc<PersistentDescriptorSet>>,
    subbuffer_allocator : SubbufferAllocator
}

impl CircleManadger {
    pub fn new(device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        let vs = circles::vs::load(device.clone()).unwrap();
        let fs = circles::fs::load(device.clone()).unwrap();
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());
        let subbuffer_allocator = SubbufferAllocator::new(memory_allocator.clone(), SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        });

        Self {
            device,
            memory_allocator,
            vs,
            fs,
            pipeline: None,
            vertex_buffer: None,
            instance_buffer: None,
            instance_size: HashMap::new(),
            instance_offset: HashMap::new(),
            descriptor_set_allocator,
            subbuffer_allocator
            // descriptor_set: HashMap::new(),
        }
    }

    pub fn clear_buffer(&mut self) {
        self.vertex_buffer = None;
        self.instance_buffer = None;
    }

    pub fn clear(&mut self) {
        self.clear_buffer();
        self.instance_offset.clear();
        // self.circles = vec![];
    }

    pub fn create_buffers<'a, I>(&mut self, plots : I) -> &mut Self
    where
        I: IntoIterator<Item = &'a Plot>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut circles = vec![];
        let mut offset = 0;
        for plot in plots {
            self.instance_offset.insert(plot.id(), offset);
            offset += plot.circles.len();
            self.instance_size.insert(plot.id(), plot.circles.len());
            circles.append(&mut plot.circles.clone());
        }

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
            circles,
        )
        .unwrap();

        self.vertex_buffer = Some(vertex_buffer);
        self.instance_buffer = Some(instance_buffer);
        self
    }

    pub fn build_pipeline(
        &mut self,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
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

        self.pipeline = Some(pipeline);
        self
    }



    pub fn draw<A>(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<A>, A>,
        plot: &Plot
    ) where
        A: CommandBufferAllocator,
    {
        let pipeline = self.pipeline.clone().unwrap();

        let uniform_buffer = {
            let buffer = self.subbuffer_allocator
                .allocate_sized()
                .unwrap();

            let uniform_data = vs::UBO {
                // projection: plot.camera.get_projection().to_cols_array_2d(),
                // modelview: plot.camera.get_view().to_cols_array_2d(),
                projection: Mat4::IDENTITY.to_cols_array_2d(),
                modelview: Mat4::IDENTITY.to_cols_array_2d(),
            };

            *buffer.write().unwrap() = uniform_data;
            buffer
        };

        let descriptor_set = create_descriptorset(pipeline.clone(), &self.descriptor_set_allocator, uniform_buffer);

        builder
            .bind_pipeline_graphics(pipeline)
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.clone().unwrap().layout().clone(),
                0,
                descriptor_set)
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone().unwrap())
            .unwrap()
            .bind_vertex_buffers(1, self.instance_buffer.clone().unwrap())
            .unwrap()
            .draw(
                self.vertex_buffer.clone().unwrap().len() as u32,
                *self.instance_size.get(&plot.id()).unwrap() as u32,
                0,
                *self.instance_offset.get(&plot.id()).unwrap() as u32,
            )
            .unwrap();
    }
}




fn create_descriptorset(
    pipeline: Arc<GraphicsPipeline>,
    descriptor_set_allocator : &StandardDescriptorSetAllocator,
    buffer: Subbuffer<impl ?Sized>) -> Arc<PersistentDescriptorSet<StandardDescriptorSetAlloc>>
{
    let pipeline_layout = pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .unwrap();

    PersistentDescriptorSet::new(
        descriptor_set_allocator,
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, buffer)], // 0 is the binding
        [],
        )
        .unwrap()
}
