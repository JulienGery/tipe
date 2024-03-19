use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::CommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, descriptor_set::{allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{RenderPass, Subpass}, shader::ShaderModule};

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

#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub local_position: [f32; 3],
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
            // layout(set = 0, binding = 0) uniform UBO
            // {
                // mat4 projection;
                // mat4 modelview;
            // } ubo;


            //out
            layout(location = 0) out vec3 o_position;
            layout(location = 1) out vec4 o_color;

            void main() {
                o_position = local_position;
                o_color = color;

                gl_Position = vec4(local_position * radius + circle_position, 1.0);
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


