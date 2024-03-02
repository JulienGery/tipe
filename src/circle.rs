use vulkano::buffer::BufferContents;
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use std::sync::Arc;
use crate::types::MyVertex;


#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct Circle {
    #[format(R32G32B32_SFLOAT)]
    pub circle_position : [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
    pub color : [f32; 4],
    #[format(R32_SFLOAT)]
    pub radius : f32,
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
        let top_right = MyVertex{ local_position : [1., 1., 0.] };
        let top_left = MyVertex{ local_position : [-1., 1., 0.] };
        let bottom_left = MyVertex{ local_position : [-1., -1., 0. ]};
        let bottom_right = MyVertex{ local_position: [1., -1., 0.] };

        vec![top_right, top_left.clone(), bottom_right.clone(), bottom_left, top_left.clone(), bottom_right.clone()]
    }

    pub fn new(radius : f32, position : [f32; 3], color : [f32; 4]) -> Self {
        Self {
            radius,
            color,
            circle_position : position
        }
    }

    pub fn get_pipeline(
        device: Arc<Device>,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
        ) -> Arc<GraphicsPipeline> {

        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        let vertex_input_state = [MyVertex::per_vertex(), Circle::per_instance()]
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
            )
            .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
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
            )
            .unwrap()
}
}
