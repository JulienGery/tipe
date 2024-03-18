use std::sync::Arc;
use std::usize;

use vulkano::{sync, Validated, VulkanError};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::GpuFuture;
use glam::Vec3;
use vulkano::command_buffer::allocator::{CommandBufferAllocator, StandardCommandBufferAllocator};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use winit::window::{WindowBuilder, Window};

use crate::circle;
use crate::plot::Plot;


// fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
//     vulkano::single_pass_renderpass!(
//         device,
//         attachments: {
//             color: {
//                 format: swapchain.image_format(), // set the format the same as the swapchain
//                 samples: 1,
//                 load_op: Clear,
//                 store_op: Store,
//             },
//         },
//         pass: {
//             color: [color],
//             depth_stencil: {},
//         },
//     )
//     .unwrap()
// }

// fn get_framebuffers(images: &[Arc<Image>], render_pass: Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
//     images
//         .iter()
//         .map(|image| {
//             let view = ImageView::new_default(image.clone()).unwrap();
//             Framebuffer::new(
//                 render_pass.clone(),
//                 FramebufferCreateInfo {
//                     attachments: vec![view],
//                     ..Default::default()
//                 },
//             )
//             .unwrap()
//         })
//         .collect::<Vec<_>>()
// }

pub struct Renderer {
    device : Arc<Device>,
    queue : Arc<Queue>,
    command_buffer_allocator : StandardCommandBufferAllocator,
    command_buffers : Vec<Arc<PrimaryAutoCommandBuffer>>,
    pub vs : Arc<ShaderModule>,
    pub fs : Arc<ShaderModule>
}

impl Renderer {
    pub fn new(device : Arc<Device>, mut queue :  Arc<Queue>) -> Self {
        // let queue = queues.next().unwrap();
        let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let command_buffers = vec![];

        let vs = circle::vs::load(device.clone()).expect("failed to create shader module");
        let fs = circle::fs::load(device.clone()).expect("failed to create shader module");

        Self {
            device,
            queue,
            command_buffer_allocator,
            command_buffers,
            vs,
            fs
        }
    }

    pub fn render_plot(&mut self, plot : &Plot) -> &mut Self {


        self
    }

    pub fn begin_render(&mut self) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit
            )
            .unwrap()
    }

    // pub fn end_render(&mut self, mut builder : AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> Arc<PrimaryAutoCommandBuffer>
    // // where A : CommandBufferAllocator,
    // {
    //     self.command_buffers[0] = builder.build().unwrap();
    //     self.command_buffers[0].clone()
    // }

    // pub fn future_render(&mut self, plot : &mut Plot) -> &mut Self {
    //     let mut builder = self.begin_render();
    //     plot.render(&mut builder);
    //     let command_buffer = self.end_render(builder);
    //
    //     let (image_i, suboptimal, acquire_futre) =
    //         match swapchain::acquire_next_image(plot.window_surface.swapchain.clone(), None).map_err(Validated::unwrap)
    //         {
    //             Ok(r) => r,
    //             Err(VulkanError::OutOfDate) => {
    //                 plot.window_surface.recreate_swapchain = true;
    //                 return self;
    //             }
    //             Err(e) => panic!("failed to acquire next image {e}"),
    //         };
    //
    //     if suboptimal {
    //         plot.recreate_swapchain = true;
    //     }
    //
    //     if let Some(image_fence) = plot.window_surface.fences[image_i as usize].clone() {
    //         image_fence.wait(None).unwrap();
    //     }
    //
    //     //todo change that
    //     let previus_future = match plot.window_surface.fences[plot.window_surface.previous_fence_i as usize].clone() {
    //         None => {
    //             let mut now = sync::now(self.device.clone());
    //             now.cleanup_finished();
    //
    //             now.boxed()
    //         }
    //
    //         Some(fence) => fence.boxed(),
    //     };
    //
    //     let future = previus_future.join(acquire_futre)
    //                                .then_execute(self.queue.clone(), command_buffer)
    //                                .unwrap()
    //                                .then_swapchain_present(
    //                                    self.queue.clone(),
    //                                    SwapchainPresentInfo::swapchain_image_index(plot.window_surface.swapchain.clone(), image_i),
    //                                 )
    //                                .then_signal_fence_and_flush();
    //
    //     plot.window_surface.fences[image_i as usize] = match future.map_err(Validated::unwrap) {
    //         Ok(value) => Some(Arc::new(value)),
    //         Err(VulkanError::OutOfDate) => {
    //             plot.window_surface.recreate_swapchain = true;
    //             None
    //         }
    //         Err(e) => {
    //             println!("failed to flush future :  {e}");
    //             None
    //         }
    //     };
    //
    //     plot.window_surface.previous_fence_i = image_i;
    //
    //     self
    // }

}
