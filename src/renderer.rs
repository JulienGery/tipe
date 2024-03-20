use std::{collections::HashMap, sync::Arc};

use vulkano::{command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, device::{Device, Queue}, memory::allocator::StandardMemoryAllocator, pipeline::graphics::viewport::Viewport};
use winit::window::WindowId;

use crate::{circle_manadger::CircleManadger, plot::Plot};

pub struct Renderer {
    // device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    // memory_allocator: Arc<StandardMemoryAllocator>,
    circles_manadger: CircleManadger,
    command_buffers: HashMap<WindowId, Vec<Arc<PrimaryAutoCommandBuffer>>>
}

impl Renderer {
    pub fn new(device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>, queue: Arc<Queue>) -> Self {
        let circles_manadger = CircleManadger::new(device.clone(), memory_allocator.clone());
        let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

        Self {
            // device,
            queue,
            command_buffer_allocator,
            // memory_allocator,
            circles_manadger,
            command_buffers: HashMap::new()
        }
    }

    pub fn create_buffer<'a, I>(&mut self, plots: I) -> &mut Self
    where
        I: IntoIterator<Item = &'a Plot>,
        I::IntoIter: ExactSizeIterator,
    {
        self.circles_manadger.create_buffers(plots);

        self
    }

    pub fn build_command_buffers(
        &mut self,
        // command_buffer_allocator: &StandardCommandBufferAllocator,
        // queue: &Arc<Queue>,
        plot: &Plot,
        // circles : &mut CircleManadger,
        viewport : Viewport,
        // descriptor_set_allocator : &StandardDescriptorSetAllocator,
        )
    {
        self.circles_manadger.build_pipeline(plot.window_surface.render_pass.clone(), viewport);
        // circles.build_pipeline(render_pass.clone(), viewport);

        let command_buffers = plot.window_surface.framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    &self.command_buffer_allocator,
                    self.queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                    )
                    .unwrap();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                        )
                    .unwrap();

                //draw here
                self.circles_manadger.draw(&mut builder, plot);

                builder.end_render_pass(Default::default())
                    .unwrap();

                builder.build().unwrap()
            })
        .collect();

        self.command_buffers
            .insert(plot.id(), command_buffers);
    }

    pub fn get_command_buffer(&mut self, plot: &Plot) -> Option<&mut Vec<Arc<PrimaryAutoCommandBuffer>>> {
        self.command_buffers.get_mut(&plot.id())
    }
}
