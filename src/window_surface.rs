use std::sync::Arc;

use vulkano::{device::{physical::PhysicalDevice, Device}, image::{view::ImageView, Image, ImageUsage}, instance::Instance, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::{Surface, Swapchain, SwapchainCreateInfo}};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::{Window, WindowBuilder, WindowId}};

pub struct WindowSurface {
    pub surface : Arc<Surface>,
    pub window: Arc<Window>,
    pub swapchain : Arc<Swapchain>,
    pub images : Vec<Arc<Image>>,
    pub render_pass : Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
}

impl WindowSurface {
    pub fn new(instance : Arc<Instance>, device : Arc<Device>, event_loop : &EventLoop<()>) -> Self {

        let window = Arc::new(WindowBuilder::new().build(event_loop).unwrap());
        let surface = Surface::from_window(instance, window.clone()).unwrap();

        let (mut swapchain, images) = {
            let caps = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .expect("failed to get surface capabilities");

            let dimension = window.inner_size();
            let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;


            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format,
                    image_extent: dimension.into(),
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
            surface,
            window,
            swapchain,
            images,
            render_pass,
            framebuffers
        }
    }


    pub fn inner_size(&self) -> PhysicalSize<u32> { self.window.inner_size() }
    pub fn id(&self) -> WindowId { self.window.id() }

    pub fn recreate_swapchain(&mut self) {

        let new_dimensions = self.inner_size();

        let (swapchain, images) = self.swapchain
                                      .recreate(SwapchainCreateInfo {
                                          image_extent: new_dimensions.into(),
                                          ..self.swapchain.create_info()
                                      })
                                     .expect("failed to recreate swapchain");

        let framebuffers = get_framebuffers(&images, self.render_pass.clone());
        self.swapchain = swapchain;
        self.framebuffers = framebuffers;
    }
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
