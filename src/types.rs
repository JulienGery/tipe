use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::buffer::BufferContents;
use glam::Mat4;

#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub local_position: [f32; 3],
}


#[derive(Clone, Debug, BufferContents)]
#[repr(C)]
pub struct Putain {
    projection : [[f32; 4]; 4],
    modelview : [[f32; 4]; 4]
}
