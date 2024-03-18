use std::sync::Arc;

use vulkano::{device::Device, instance::Instance, memory::allocator::StandardMemoryAllocator};
use winit::{event_loop::EventLoop, window::WindowId};

use crate::{circles::{Circle, CircleManadger}, window_surface::WindowSurface};

pub struct Plot {
    pub window_surface : WindowSurface,
    pub circles : CircleManadger,
}

impl Plot {
    pub fn new(instance : Arc<Instance>, device : Arc<Device>, event_loop : &EventLoop<()>, memory_allocator : Arc<StandardMemoryAllocator>) -> Self {

        let window_surface = WindowSurface::new(instance.clone(), device.clone(), event_loop);
        let circles = CircleManadger::new(device.clone(), memory_allocator.clone());

        Self {
            window_surface,
            circles
        }
    }

    pub fn id(&self) -> WindowId {
        self.window_surface.id()
    }

    pub fn scatter(&mut self, data : &mut Vec<Circle>) -> &mut Self {
        self.circles.append(data);
        self
    }

    pub fn create_buffer(&mut self) -> &mut Self {
        self.circles.create_buffers();
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.circles.clear();
        self
    }
}
