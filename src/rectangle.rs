// use std::sync::Arc;
//
// // use vulkano::command_buffer::{allocator::CommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
// use vulkano::memory::allocator::StandardMemoryAllocator;
// // use vulkano::device::Device;
//
// use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
// use vulkano::descriptor_set::persistent::PersistentDescriptorSet;
// use vulkano::descriptor_set::WriteDescriptorSet;
// use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
// use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
// use vulkano::command_buffer::{
//     allocator::CommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
// };
// use vulkano::device::Device;
// use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
// use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
// use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
// use vulkano::pipeline::graphics::multisample::MultisampleState;
// use vulkano::pipeline::graphics::rasterization::RasterizationState;
// use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
// use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
// use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
// use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
// use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
// use vulkano::shader::ShaderModule;
//
//
// #[derive(BufferContents, Vertex, Clone, Debug)]
// #[repr(C)]
// pub struct Rectangle {
//     #[format(R32G32B32_SFLOAT)]
//     pub bottom_right: [f32; 3],
//     pub top_left: [f32; 3]
//     #[format(R32G32B32A32_SFLOAT)]
//     pub color: [f32; 4],
//     #[format(R32_SFLOAT)]
//     pub height: f32,
// }
//
// impl Rectangle {
//     pub fn new(rectangle_position : [f32; 3], color : [f32; 4], height : f32) -> Self {
//
//         Self {
//             rectangle_position,
//             color,
//             height
//         }
//     }
// }
//
//
// pub struct RectangleManadger {
//     device : Arc<Device>,
//     memory_allocator : Arc<StandardMemoryAllocator>,
//     rectangles : Vec<Rectangle>
// }
//
//
// impl RectangleManadger {
//     pub fn new(device : Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
//
//         Self {
//             device,
//             memory_allocator,
//             rectangles : vec![]
//         }
//     }
//
//
//
//
//     pub fn bake(&mut self) -> &mut Self {
//
//         let vertex_buffer = Buffer::from_iter(
//             self.memory_allocator.clone(),
//             BufferCreateInfo {
//                 usage: BufferUsage::VERTEX_BUFFER,
//                 ..Default::default()
//             },
//             AllocationCreateInfo {
//                 memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
//                     | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
//                 ..Default::default()
//             },
//             ::vertex(),
//         )
//         .unwrap();
//
//         let instance_buffer = Buffer::from_iter(
//             self.memory_allocator.clone(),
//             BufferCreateInfo {
//                 usage: BufferUsage::VERTEX_BUFFER,
//                 ..Default::default()
//             },
//             AllocationCreateInfo {
//                 memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
//                     | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
//                 ..Default::default()
//             },
//             self.circles.clone(),
//         )
//         .unwrap();
//
//         // let uniform_buffer = SubbufferAllocator::new(
//         //     self.memory_allocator.clone(),
//         //     SubbufferAllocatorCreateInfo {
//         //         buffer_usage: BufferUsage::UNIFORM_BUFFER,
//         //         memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
//         //             | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
//         //             ..Default::default()
//         //     },
//         //     );
//
//         self.vertex_buffer = Some(vertex_buffer);
//         self.instance_buffer = Some(instance_buffer);
//         self
//     }
//
//
//     pub fn render<A>(&mut self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<A>, A>) -> &mut Self
//     where A: CommandBufferAllocator,
//     {
//
//         self
//     }
// }
//
//
