use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents};
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::swapchain::Surface;
use vulkano::{device::Device, instance::Instance};
use vulkano::memory::allocator::StandardMemoryAllocator;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};

use crate::{camera::Camera, circle::{Circle, CircleManadger}, window_surface::WindowSurface};

pub struct Plot {
    pub window_surface : WindowSurface,
    pub circles : CircleManadger,
    // rectangles : RectangleManadger,
    // lines : LineManadger,
    pub camera : Camera,
    pub is_bake : bool,
    pub recreate_swapchain : bool
}


impl Plot {
    pub fn new(instance : Arc<Instance>, device : Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>, event_loop : &EventLoop<()>) -> Self {
        let surface = WindowSurface::new(instance.clone(), device.clone(), event_loop);

        Self::from(instance, device, memory_allocator, event_loop, surface)
    }

    pub fn from(instance : Arc<Instance>, device : Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>, event_loop : &EventLoop<()>, window_surface : WindowSurface) -> Self {

        let circles = CircleManadger::new(device, memory_allocator);
        // let rectangles = RectangleManadger::new(device, memory_allocator);
        // let lines = LineManadger::new(device, memory_allocator);
        let camera = Camera::new();


        Self {
            window_surface,
            circles,
            // rectangles,
            // lines,
            camera,
            is_bake : false,
            recreate_swapchain : false
        }
    }

    pub fn id(&self) -> WindowId {
        self.window_surface.id()
    }

    pub fn bake(&mut self) -> &mut Self {
        self.circles.bake();
        // self.rectangles.bake();
        // self.lines.bake();

        self.is_bake = true;
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.circles.clear();
        // self.lines.clear();
        // self.rectangles.clear();

        self
    }

    pub fn clear_buffer(&mut self) -> &mut Self {
        self.circles.clear_buffer();


        self
    }

    pub fn scatter(&mut self, x : Vec<f32>, y : Vec<f32>, radius : f32, color : [f32; 4]) -> &mut Self {
        let mut circles = x.iter()
                          .zip(y.iter())
                          .map(|(a, b)| Circle::new(radius, [*a, *b, 0.], color))
                          .collect();

        self.circles.append(&mut circles);
        self
    }


    // pub fn show(&mut self) -> &mut Self {
    //     if !self.is_bake { self.bake(); }
    //
    //     self
    // }

    pub fn render<A>(&mut self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<A>, A>) -> &mut Self
    where A: CommandBufferAllocator,
    {
        builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values : vec![Some([1.0, 0.0, 1.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(self.window_surface.framebuffers[0].clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
            )
            .unwrap();

        self.circles.render(builder);

        //end render plot
        builder.end_render_pass(Default::default());

        self
    }
}
