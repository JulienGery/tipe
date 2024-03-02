use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::buffer::BufferContents;



#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub local_position: [f32; 3],
}
