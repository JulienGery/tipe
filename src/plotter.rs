use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{WindowBuilder, Window};

use crate::circle::{self, Circle};
use crate::types::{self, MyVertex};

// fn get_pipeline(
//     device: Arc<Device>,
//     vs: Arc<ShaderModule>,
//     fs: Arc<ShaderModule>,
//     render_pass: Arc<RenderPass>,
//     viewport: Viewport,
// ) -> Arc<GraphicsPipeline> {
//
//     let vs = vs.entry_point("main").unwrap();
//     let fs = fs.entry_point("main").unwrap();
//
//     let vertex_input_state = [types::MyVertex::per_vertex(), Circle::per_instance()]
//         .definition(&vs.info().input_interface)
//         .unwrap();
//
//     let stages = [
//         PipelineShaderStageCreateInfo::new(vs),
//         PipelineShaderStageCreateInfo::new(fs),
//     ];
//
//     let layout = PipelineLayout::new(
//         device.clone(),
//         PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
//             .into_pipeline_layout_create_info(device.clone())
//             .unwrap(),
//     )
//     .unwrap();
//
//     let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
//
//     GraphicsPipeline::new(
//         device.clone(),
//         None,
//         GraphicsPipelineCreateInfo {
//             stages: stages.into_iter().collect(),
//             vertex_input_state: Some(vertex_input_state),
//             input_assembly_state: Some(InputAssemblyState::default()),
//             viewport_state: Some(ViewportState {
//                 viewports: [viewport].into_iter().collect(),
//                 ..Default::default()
//             }),
//             rasterization_state: Some(RasterizationState::default()),
//             multisample_state: Some(MultisampleState::default()),
//             color_blend_state: Some(ColorBlendState::with_attachment_states(
//                 subpass.num_color_attachments(),
//                 ColorBlendAttachmentState::default(),
//             )),
//             subpass: Some(subpass.into()),
//             ..GraphicsPipelineCreateInfo::layout(layout)
//         },
//     )
//     .unwrap()
// }

fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &[Arc<Framebuffer>],
    vertex_buffer: &Subbuffer<[MyVertex]>,
    instance_buffer : &Subbuffer<[Circle]>
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
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
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .bind_vertex_buffers(1, instance_buffer.clone())
                .unwrap()
                .draw(vertex_buffer.len() as u32, instance_buffer.len() as u32, 0, 0)
                .unwrap()
                .end_render_pass(Default::default())
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
    event_loop : EventLoop<()>,
    instance : Arc<Instance>,
    window : Arc<Window>,
    surface : Arc<Surface>,
    physical_device : Arc<PhysicalDevice>,
    device : Arc<Device>,
    queue : Arc<Queue>,
    render_pass : Arc<RenderPass>,
    framebuffers : Vec<Arc<Framebuffer>>,
    images : Vec<Arc<Image>>,
    swapchain : Arc<Swapchain>,
    memory_allocator : Arc<StandardMemoryAllocator>,
    circles : Vec<Circle>,
    vs : Arc<ShaderModule>,
    fs : Arc<ShaderModule>,
    is_bake : bool,
    vertex_buffer : Option<Subbuffer<[MyVertex]>>,
    instance_buffer : Option<Subbuffer<[Circle]>>
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
        let vs = circle::vs::load(device.clone()).expect("failed to create shader module");
        let fs = circle::fs::load(device.clone()).expect("failed to create shader module");

        Self {
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
            circles : vec![],
            vs,
            fs,
            is_bake : false,
            vertex_buffer : None,
            instance_buffer : None,
        }
    }

    pub fn bake(&mut self) -> &mut Self {
        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
            },
            Circle::vertex(),
            )
            .unwrap();

        let instance_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage : BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter : MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.circles.clone(),
            )
            .unwrap();


        self.instance_buffer = Some(instance_buffer);
        self.vertex_buffer = Some(vertex_buffer);
        self.is_bake = true;
        self
    }

    pub fn show(&mut self) {
        if !self.is_bake {
            self.bake();
        }

        let mut viewport = Viewport {
            offset: [0.0, 0.0],
            extent: self.window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let pipeline = Circle::get_pipeline(
            self.device.clone(),
            self.vs.clone(),
            self.fs.clone(),
            self.render_pass.clone(),
            viewport.clone(),
            );

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(self.device.clone(), Default::default());

        let mut command_buffers = get_command_buffers(
            &command_buffer_allocator,
            &self.queue,
            &pipeline,
            &self.framebuffers,
            &self.vertex_buffer.clone().unwrap(),
            &self.instance_buffer.clone().unwrap()
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
                        let new_pipeline = Circle::get_pipeline(
                            self.device.clone(),
                            self.vs.clone(),
                            self.fs.clone(),
                            self.render_pass.clone(),
                            viewport.clone(),
                            );
                        command_buffers = get_command_buffers(
                            &command_buffer_allocator,
                            &self.queue,
                            &new_pipeline,
                            &new_framebuffers,
                            &self.vertex_buffer.clone().unwrap(),
                            &self.instance_buffer.clone().unwrap()
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

        // self.window.set_visible(false);
    }

    pub fn scatter(&mut self, x : &Vec<f32>, y : &Vec<f32>, radius : f32, color : [f32; 4]) -> &mut Self {
        assert!(x.len() == y.len());

        self.circles = x.iter().zip(y.iter()).map(|(a, b)| Circle::new(radius, [*a, *b, 0.], color)).collect();
        self
    }
}
