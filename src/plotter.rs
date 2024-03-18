use std::collections::HashMap;
use std::sync::Arc;

use glam::Vec3;
use vulkano::command_buffer::{sys::CommandBufferBeginInfo, CommandBufferLevel};
use vulkano::memory::allocator::{MemoryTypeFilter, AllocationCreateInfo};
use vulkano::buffer::{Buffer, BufferCreateInfo};
use vulkano::buffer::Subbuffer;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::{DescriptorBindingFlags, DescriptorSetLayoutBinding, DescriptorType};
use vulkano::descriptor_set::pool::{DescriptorPool, DescriptorSetAllocateInfo};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::shader::{ShaderStage, ShaderStages};
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture, PipelineStages};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder, WindowId};

use crate::camera::{self, Camera, CameraUniformBuffer};
use crate::circle::{vs, Circle, CircleManadger};
use crate::plot::Plot;
use crate::renderer::Renderer;
use crate::window_surface::WindowSurface;
use crate::{renderer, types};


//maybe should not build pipeline
// fn get_command_buffers(
//     command_buffer_allocator: &StandardCommandBufferAllocator,
//     queue: &Arc<Queue>,
//     framebuffers: &[Arc<Framebuffer>],
//     circles : &mut CircleManadger,
//     render_pass: Arc<RenderPass>,
//     viewport : Viewport,
//     descriptor_set_allocator : &StandardDescriptorSetAllocator,
//     buffer: Subbuffer<impl ?Sized>
// ) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
//     circles.build_pipeline(render_pass.clone(), viewport, descriptor_set_allocator, buffer);
//
//     framebuffers
//         .iter()
//         .map(|framebuffer| {
//             let mut builder = AutoCommandBufferBuilder::primary(
//                 command_buffer_allocator,
//                 queue.queue_family_index(),
//                 CommandBufferUsage::MultipleSubmit,
//                 )
//                 .unwrap();
//
//             builder
//                 .begin_render_pass(
//                     RenderPassBeginInfo {
//                         clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
//                         ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
//                     },
//                     SubpassBeginInfo {
//                         contents: SubpassContents::Inline,
//                         ..Default::default()
//                     },
//                     )
//                 .unwrap();
//
//             //draw here
//             circles.draw(&mut builder);
//
//             builder.end_render_pass(Default::default())
//                 .unwrap();
//
//             builder.build().unwrap()
//         })
//         .collect()
// }

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available")
}


pub struct Plotter {
    library : Arc<VulkanLibrary>,
    instance : Arc<Instance>,
    physical_device : Arc<PhysicalDevice>,
    device : Arc<Device>,
    event_loop : EventLoop<()>,
    memory_allocator : Arc<StandardMemoryAllocator>,
    descriptor_set_allocator : StandardDescriptorSetAllocator,
    renderer : Renderer,
    pub plots : HashMap<WindowId, Plot>,
    current_window : WindowId,
    queue : Arc<Queue>
}


impl Plotter {
    pub fn new() -> Self {
        let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");

        let event_loop = EventLoop::new();
        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
            )
            .expect("failed to create instance");

        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = select_physical_device(&instance, &surface, &device_extensions);

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions, // new
                ..Default::default()
            },
            )
            .expect("failed to create device");

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());

        let queue = queues.next().unwrap();

        let window_surface = WindowSurface::new(instance.clone(), device.clone(), &event_loop);
        let renderer = Renderer::new(device.clone(), queue.clone());

        let plot = Plot::from(instance.clone(), device.clone(), memory_allocator.clone(), &event_loop, window_surface);
        let current_window = plot.id();
        let mut plots = HashMap::new();
        plots.insert(plot.id(), plot);

        Self {
            library,
            instance,
            physical_device,
            device,
            event_loop,
            memory_allocator,
            descriptor_set_allocator,
            renderer,
            plots,
            current_window,
            queue
        }
    }

    pub fn bake(&mut self) -> &mut Self {
        // self.circles.bake();

        for plot in self.plots.values_mut() {
            plot.bake();
        }

        self
    }

    pub fn clear(&mut self) -> &mut Self {
        // self.circles.clear();

        self.plots.clear();
        let plot = Plot::new(self.instance.clone(), self.device.clone(), self.memory_allocator.clone(), &self.event_loop);
        self.plots.insert(plot.id(), plot);

        self
    }

    pub fn show(&mut self) -> &mut Self {
        self.bake(); // create buffers

        self.main_loop(); // main loop

        self.clear(); // destroy data
        self
    }

    pub fn scatter(&mut self, x : Vec<f32>, y : Vec<f32>, radius : f32, color : [f32; 4]) -> &mut Self {
        assert!(x.len() == y.len());

        self.plots.get_mut(&self.current_window)
                  .unwrap()
                  .scatter(x, y, radius, color);

        self
    }

    // fn render(&mut self, window_id : WindowId) -> &mut Self {
    //     let plot = self.plots.get_mut(&window_id).unwrap();
    //     self.renderer.future_render(plot);
    //
    //     self
    // }

    fn main_loop(&mut self) {
        let mut viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [0.0, 0.0],
            depth_range: 0.0..=1.0,
        };

        let command_buffer_allocator = StandardCommandBufferAllocator::new(self.device.clone(), Default::default());
        let plot = self.plots.get(&self.current_window).unwrap();

        let mut command_buffer = get_command_buffers(
            &command_buffer_allocator,
            &self.queue,
            &plot.window_surface.framebuffers,
            &mut plot.circles,
            plot.window_surface.render_pass.clone(),
            viewport,
            &self.descriptor_set_allocator,
            &self.renderer
            );


        self.event_loop.run_return(move |event, _, control_flow| {
            // cont.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                }
                Event::RedrawRequested(window_id) => {


                    // renderer.future_render(self.plots.get_mut(&window_id).unwrap());
                }
                _ => (),
            }
        });
    }
}


fn get_render_pass(device : Arc<Device>, swapchain : Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments : {
            color : {
                format : swapchain.image_format(),
                samples : 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass : {
            color : [color],
            depth_stencil : {},
        },
    )
    .unwrap()
}

fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    framebuffers: &[Arc<Framebuffer>],
    circles : &mut CircleManadger,
    render_pass: Arc<RenderPass>,
    viewport : Viewport,
    descriptor_set_allocator : &StandardDescriptorSetAllocator,
    // buffer: Subbuffer<impl ?Sized>,
    renderer : &Renderer
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    circles.build_pipeline(render_pass.clone(), viewport, descriptor_set_allocator, renderer);

    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
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
            circles.draw(&mut builder);

            builder.end_render_pass(Default::default())
                .unwrap();

            builder.build().unwrap()
        })
        .collect()
}
