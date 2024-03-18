use std::sync::Arc;

use vulkano::swapchain::{PresentFuture, SwapchainAcquireFuture};
use vulkano::sync::future::JoinFuture;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::device::Device;
use vulkano::instance::Instance;
use vulkano::sync::future::FenceSignalFuture;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder, WindowId};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::sync::GpuFuture;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::RenderPass;
use vulkano::image::view::ImageView;
use vulkano::image::Image;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::image::ImageUsage;

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

// Box<dyn GpuFuture>
// Vec<Option<Arc<FenceSignalFuture<Box<dyn GpuFuture>>>>
pub struct WindowSurface {
    pub device : Arc<Device>,
    pub window : Arc<Window>,
    pub surface : Arc<Surface>,
    pub images : Vec<Arc<Image>>,
    pub swapchain : Arc<Swapchain>,
    pub framebuffers : Vec<Arc<Framebuffer>>,
    pub recreate_swapchain : bool,
    pub previous_frame_end : Option<Box<dyn GpuFuture>>,
    pub fences : Vec<Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>>>>,
    pub previous_fence_i : u32,
    pub render_pass : Arc<RenderPass>
}

impl WindowSurface {
    pub fn new(instance : Arc<Instance>, device : Arc<Device>, event_loop : &EventLoop<()>) -> Self {
        let window = Arc::new(WindowBuilder::new().build(event_loop).unwrap());
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        Self::from(instance, device, event_loop, window, surface)
    }

    pub fn from(instance : Arc<Instance>, device: Arc<Device>, event_loop : &EventLoop<()>, window : Arc<Window>, surface : Arc<Surface>) -> Self {
        let image_format = device.physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        // let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        let (mut swapchain, images) = create_swapchain(&device, &surface, &window);
        let render_pass = get_render_pass(device.clone(), swapchain.clone());
        let framebuffers = get_framebuffers(&images, render_pass.clone());

        // let mut fences: Vec<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>> = vec![None; 2];

        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; images.len()];

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments : {
                color: {
                    format: swapchain.image_format(),
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
            .unwrap();

        Self {
            device,
            window,
            surface,
            images,
            swapchain,
            framebuffers,
            recreate_swapchain : false,
            previous_frame_end : None,
            fences,
            previous_fence_i : 0,
            render_pass
        }
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn frames_in_flight(&self) -> u32 {
        self.swapchain.image_count()
    }

    pub fn recreate_swapchain(&mut self, window : Arc<Window>) {
        let (swapchain, images) = create_swapchain(&self.device, &self.surface, &window);

        self.swapchain = swapchain;
    }
}


fn create_swapchain(device : &Arc<Device>, surface : &Arc<Surface>, window : &Arc<Window>) -> (Arc<Swapchain>, Vec<Arc<Image>>){

    let caps = device.physical_device()
        .surface_capabilities(surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = window.inner_size();
    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format = device.physical_device()
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
}
