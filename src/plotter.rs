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

use crate::camera::{self, Camera, CameraUniformBuffer};
use crate::circle::{vs, Circle, CircleManadger};
use crate::types;


//maybe should not build pipeline
fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    framebuffers: &[Arc<Framebuffer>],
    circles : &mut CircleManadger,
    render_pass: Arc<RenderPass>,
    viewport : Viewport,
    descriptor_set_allocator : &StandardDescriptorSetAllocator,
    buffer: Subbuffer<impl ?Sized>
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    circles.build_pipeline(render_pass.clone(), viewport, descriptor_set_allocator, buffer);

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

pub struct Plot {
    library : Arc<VulkanLibrary>,
    event_loop : EventLoop<()>, //render class ?
    instance : Arc<Instance>,
    window : Arc<Window>, //render class
    surface : Arc<Surface>, //render class
    physical_device : Arc<PhysicalDevice>,
    device : Arc<Device>,
    queue : Arc<Queue>,
    render_pass : Arc<RenderPass>, //render class
    framebuffers : Vec<Arc<Framebuffer>>, //render class
    images : Vec<Arc<Image>>, //render class
    swapchain : Arc<Swapchain>, //render class
    memory_allocator : Arc<StandardMemoryAllocator>,
    is_bake : bool,
    circles : CircleManadger, //render class
    camera : Camera, //render class
    descriptor_set_allocator : StandardDescriptorSetAllocator,
    uniform_buffer : Subbuffer<[CameraUniformBuffer]> //render class
}


impl Plot {
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
        window.set_visible(false);

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

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let mut camera = Camera::new();
        camera.set_position(Vec3::new(0., 0., -5.))
              .set_rotation(Vec3::Z)
              .set_perspective(60., 1920./1080., 0.1, 256.);

        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone(), Default::default());


        let uniform_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            [camera.get_data()],
        )
        .unwrap();

        Self {
            circles : CircleManadger::new(device.clone(), memory_allocator.clone()),
            library,
            event_loop,
            instance,
            window,
            surface,
            physical_device,
            device,
            queue,
            render_pass,
            framebuffers,
            images,
            swapchain,
            memory_allocator,
            camera,
            is_bake : false,
            descriptor_set_allocator,
            uniform_buffer
        }
    }

    pub fn bake(&mut self) -> &mut Self {
        self.circles.bake();

        self.is_bake = true;
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.circles.clear();

        self
    }

    pub fn show(&mut self) -> &mut Self {
        if !self.is_bake {
            self.bake();
        }

        self.main_loop();
        self.clear();
        self
    }

    pub fn scatter(&mut self, x : &Vec<f32>, y : &Vec<f32>, radius : f32, color : [f32; 4]) -> &mut Self {
        assert!(x.len() == y.len());

        let mut circles =  x.iter()
                            .zip(y.iter())
                            .map(|(a, b)| Circle::new(radius, [*a, *b, 0.], color))
                            .collect();

        self.circles.append(&mut circles);
        self
    }

    fn update_uniform_buffer(&mut self) {
        let mut content = self.uniform_buffer.write().unwrap();
        content[0] = self.camera.get_data();
    }

    fn main_loop(&mut self) {
        let mut viewport = Viewport {
            offset: [0.0, 0.0],
            extent: self.window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(self.device.clone(), Default::default());

        let mut command_buffers = get_command_buffers(
            &command_buffer_allocator,
            &self.queue,
            &self.framebuffers,
            &mut self.circles,
            self.render_pass.clone(),
            viewport.clone(),
            &self.descriptor_set_allocator,
            self.uniform_buffer.clone()
            );


        self.window.set_visible(true);

        let mut window_resized = false;
        let mut recreate_swapchain = false;

        let frames_in_flight = self.images.len();
        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let mut previous_fence_i = 0;

        self.event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                window_resized = true;
            }
            Event::MainEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;

                    let new_dimensions = self.window.inner_size();

                    let (new_swapchain, new_images) = self.swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: new_dimensions.into(),
                            ..self.swapchain.create_info()
                        })
                    .expect("failed to recreate swapchain");

                    self.swapchain = new_swapchain;
                    let new_framebuffers = get_framebuffers(&new_images, self.render_pass.clone());

                    if window_resized {
                        window_resized = false;

                        viewport.extent = new_dimensions.into();

                        command_buffers = get_command_buffers(
                            &command_buffer_allocator,
                            &self.queue,
                            &new_framebuffers,
                            &mut self.circles,
                            self.render_pass.clone(),
                            viewport.clone(),
                            &self.descriptor_set_allocator,
                            self.uniform_buffer.clone()
                            );
                    }
                }

                let (image_i, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(self.swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                    {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                if let Some(image_fence) = &fences[image_i as usize] {
                    image_fence.wait(None).unwrap();
                }

                let previous_future = match fences[previous_fence_i as usize].clone() {
                    // Create a NowFuture
                    None => {
                        let mut now = sync::now(self.device.clone());
                        now.cleanup_finished();

                        now.boxed()
                    }
                    // Use the existing FenceSignalFuture
                    Some(fence) => fence.boxed(),
                };

                let future = previous_future
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffers[image_i as usize].clone())
                    .unwrap()
                    .then_swapchain_present(
                        self.queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
                        )
                    .then_signal_fence_and_flush();

                fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                    Ok(value) => Some(Arc::new(value)),
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        None
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        None
                    }
                };

                previous_fence_i = image_i;
            }
            _ => (),
        });

        self.window.set_visible(false);
    }
}
