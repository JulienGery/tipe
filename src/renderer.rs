use std::sync::Arc;
use glam::Vec3;
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
use winit::window::{WindowBuilder, Window};

use crate::camera::{Camera, CameraUniformBuffer};
use crate::circle::CircleManadger;

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

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(), // set the format the same as the swapchain
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap()
}

fn get_framebuffers(images: &[Arc<Image>], render_pass: Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
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

pub struct Renderer {
    // event_loop : EventLoop<()>, //render class ?
    // window : Arc<Window>, //render class
    // surface : Arc<Surface>, //render class
    device : Arc<Device>,
    queue : Arc<Queue>,
    // render_pass : Arc<RenderPass>, //render class
    // framebuffers : Vec<Arc<Framebuffer>>, //render class
    images : Vec<Arc<Image>>, //render class
    swapchain : Arc<Swapchain>, //render class
}

impl Renderer {
    pub fn new(device : Arc<Device>, event_loop : &EventLoop<()>, window : Arc<Window>) -> Self {
        let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
            )
            .expect("failed to create instance");

        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            select_physical_device(&instance, &surface, &device_extensions);

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

        let queue = queues.next().unwrap();

        let (mut swapchain, images) = {
            let caps = physical_device
                .surface_capabilities(&surface, Default::default())
                .expect("failed to get surface capabilities");

            let dimensions = window.inner_size();
            let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
            let image_format = physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format,
                    image_extent: dimensions.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha,
                    ..Default::default()
                },
                )
                .unwrap()
        };

        let render_pass = get_render_pass(device.clone(), swapchain.clone());
        let framebuffers = get_framebuffers(&images, render_pass.clone());

        Self {
            device,
            images,
            queue,
            swapchain
        }
    }
}
