use core::panic;
use std::{collections::HashMap, sync::Arc};

use vulkano::{command_buffer::{self, allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::allocator::StandardDescriptorSetAllocator, device::{self, physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags}, instance::{Instance, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::graphics::viewport::Viewport, render_pass::{Framebuffer, RenderPass}, swapchain::{self, Surface, SwapchainPresentInfo}, sync::{future::FenceSignalFuture, GpuFuture}, Validated, VulkanError};
use winit::{event::{Event, WindowEvent}, platform::run_return::EventLoopExtRunReturn, window::{WindowBuilder, WindowId}};
use winit::event_loop::{ControlFlow, EventLoop};
use crate::{circles::{Circle, CircleManadger}, plot::Plot, window_surface::WindowSurface};

pub struct Plotter {
    instance : Arc<Instance>,
    device : Arc<Device>,
    event_loop : EventLoop<()>,
    plots : HashMap<WindowId, Plot>,
    // plot: Plot,
    //replace with plot
    // window_surface : WindowSurface,
    // circles : CircleManadger,
    queue:  Arc<Queue>,
    current_plot: WindowId,
    memory_allocator : Arc<StandardMemoryAllocator>
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

impl Plotter {
    pub fn new() -> Self {

        let library = vulkano::VulkanLibrary::new().expect("no local vulkan library/DLL");
        let event_loop = EventLoop::new();

        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            }
        ).expect("failed to create instance");

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
                enabled_extensions: device_extensions,
                ..Default::default()
            }
            )
            .expect("failed to create device");

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());
        let circles = CircleManadger::new(device.clone(), memory_allocator.clone());

        let plot = Plot::new(instance.clone(), device.clone(), &event_loop, memory_allocator.clone());
        let current_plot = plot.id();
        let mut plots = HashMap::new();
        plots.insert(current_plot, plot);

        Self {
            instance,
            device,
            event_loop,
            plots,
            queue,
            current_plot,
            memory_allocator
        }
    }

    pub fn current_plot(&mut self) -> &mut Plot {
        self.plots.get_mut(&self.current_plot).unwrap()
    }

    pub fn scatter(&mut self, x : Vec<f32>, y : Vec<f32>, radius : f32, color : [f32; 4]) -> &mut Self {
        let mut circles = x
            .iter()
            .zip(y.iter())
            .map(|(a, b)| Circle::new(radius, [*a, *b, 0.], color))
            .collect();

        self.current_plot()
            .scatter(&mut circles);
        self
    }

    //rename to clean
    pub fn clear(&mut self) -> &mut Self {
        for plot in self.plots.values_mut() {
            plot.clear();
        }

        self
    }

    pub fn new_plot(&mut self) -> &mut Self {
        let plot = Plot::new(
            self.instance.clone(),
            self.device.clone(),
            &self.event_loop,
            self.memory_allocator.clone()
            );

        self.current_plot = plot.id();
        self.plots.insert(plot.id(), plot);
        self
    }

    pub fn create_buffers(&mut self) -> &mut Self {
        for plot in self.plots.values_mut() {
            plot.create_buffer();
        }

        self
    }

    pub fn show(&mut self) -> &mut Self {
        self.create_buffers();

        self.main_loop();

        self.clear()
    }

    fn main_loop(&mut self)  -> &mut Self{
        //does not work if not borrowed like this
        let plot = self.plots.get_mut(&self.current_plot).unwrap();

        //need to be moved to window_surface
        let mut viewport = Viewport {
            offset: [0., 0.],
            extent: plot.window_surface.inner_size().into(),
            depth_range: 0.0..=1.0,
        };


        let command_buffer_allocator = StandardCommandBufferAllocator::new(self.device.clone(), Default::default());

        let mut command_buffers = get_command_buffers(
            &command_buffer_allocator,
            &self.queue,
            &plot.window_surface.framebuffers,
            &mut plot.circles,
            plot.window_surface.render_pass.clone(),
            viewport.clone()
            );

        let mut window_resized = false;
        let mut recreate_swapchain = false;

        let frames_in_flight = plot.window_surface.images.len();
        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let mut previous_fence_i = 0;

        self.event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent {
                // window_id,
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
            Event::RedrawRequested(window_id) => println!("window_id : {:?}", window_id),
            Event::MainEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;

                    plot.window_surface.recreate_swapchain();
                    if window_resized {
                        window_resized = false;

                        viewport.extent = plot.window_surface.inner_size().into();

                        command_buffers = get_command_buffers(
                            &command_buffer_allocator,
                            &self.queue,
                            &plot.window_surface.framebuffers,
                            &mut plot.circles,
                            plot.window_surface.render_pass.clone(),
                            viewport.clone()
                            );
                    }

                }

                let (image_i, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(plot.window_surface.swapchain.clone(), None).map_err(Validated::unwrap)
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

                if let Some(image_fence) = &fences[image_i as usize] {
                    image_fence.wait(None).unwrap();
                }

                let previous_future = match fences[previous_fence_i as usize].clone() {

                    None => {
                        let mut now = vulkano::sync::now(self.device.clone());
                        now.cleanup_finished();

                        now.boxed()
                    }

                    Some(fence) => fence.boxed(),
                };

                let future = previous_future
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffers[image_i as usize].clone())
                    .unwrap()
                    .then_swapchain_present(
                        self.queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(plot.window_surface.swapchain.clone(), image_i),
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

        self
    }
}



fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    framebuffers: &[Arc<Framebuffer>],
    circles : &mut CircleManadger,
    render_pass: Arc<RenderPass>,
    viewport : Viewport,
    // descriptor_set_allocator : &StandardDescriptorSetAllocator,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    circles.build_pipeline(render_pass.clone(), viewport);

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
