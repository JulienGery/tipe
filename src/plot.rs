use std::sync::Arc;

use glam::Vec3;
use vulkano::{device::Device, instance::Instance, memory::allocator::StandardMemoryAllocator};
use winit::{event_loop::EventLoop, window::WindowId};

use crate::{camera::Camera, circles::Circle, window_surface::WindowSurface};

pub struct Plot {
    pub window_surface : WindowSurface,
    pub circles: Vec<Circle>,
    pub camera: Camera,
}

impl Plot {
    pub fn new(instance : Arc<Instance>, device : Arc<Device>, event_loop : &EventLoop<()>) -> Self {
        let window_surface = WindowSurface::new(instance.clone(), device.clone(), event_loop);
        let mut camera = Camera::new();
        camera.set_position(Vec3::new(0., 0., -1.));
        camera.set_rotation(Vec3::NEG_Z);

        Self {
            window_surface,
            camera,
            circles: vec![]
        }
    }

    pub fn id(&self) -> WindowId {
        self.window_surface.id()
    }

    pub fn scatter(&mut self, data : &mut Vec<Circle>) -> &mut Self {
        self.circles.append(data);
        self
    }

    // pub fn create_buffer(&mut self) -> &mut Self {
    //     self.circles.create_buffers();
    //     self
    // }

    pub fn clear(&mut self) -> &mut Self {
        self.circles.clear();
        self
    }

    pub fn height(&self) -> u32 { self.window_surface.inner_size().height }
    pub fn width(&self) -> u32 { self.window_surface.inner_size().width }
    pub fn aspect(&self) -> f32 { self.width() as f32 / self.height() as f32 }

    pub fn update_camera(&mut self, fov: f32, znear: f32, zfar: f32) -> &mut Self {
        self.camera.set_perspective(fov, self.aspect(), znear, zfar);
        self
    }
}
