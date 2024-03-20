use std::f32::consts::PI;

use vulkano::buffer::BufferContents;
use glam::{Mat4, Vec4, Vec3};

pub struct Camera {
    position : Vec3,
    rotation : Vec3,
    view_pos : Vec4,
    perspective : Mat4,
    view : Mat4,
    fov : f32,
    znear : f32,
    zfar : f32,
    updated : bool
}

fn to_rad(theta : f32) -> f32 {
    theta * PI / 180.0
}


#[derive(Clone, Debug, BufferContents)]
#[repr(C)]
pub struct CameraUniformBuffer {
    projection : [[f32; 4]; 4],
    modelview : [[f32; 4]; 4]
}

impl Camera {
    pub fn new() -> Self {
        let position = Vec3::new(0., 0., 0.);
        let rotation = position.clone();
        let view_pos = Vec4::new(0., 0., 0., 1.);
        let perspective = Mat4::IDENTITY;
        let view = Mat4::IDENTITY;

        Self {
            position,
            rotation,
            view_pos,
            perspective,
            updated : true,
            fov : 20.,
            znear : 0.,
            zfar : 1.,
            view
        }
    }

    pub fn set_position(&mut self, position : Vec3) -> &mut Self {
        self.position = position;
        self.update_view_matrix();
        self
    }

    pub fn set_rotation(&mut self, rotation : Vec3) -> &mut Self {
        self.rotation = rotation;
        self.update_view_matrix();
        self
    }

    pub fn set_perspective(&mut self, fov : f32, aspect : f32, znear : f32, zfar : f32) -> &mut Self {
        let current_matrix = self.perspective;
        self.fov = fov;
        self.znear = znear;
        self.zfar = zfar;
        self.perspective = Mat4::perspective_rh(to_rad(fov), aspect, znear, zfar);

        if current_matrix != self.perspective {
            self.updated = true;
        }

        self
    }

    pub fn update_aspect_ratio(&mut self, aspect : f32) -> &mut Self {
        let current_matrix = self.perspective;
        self.perspective = Mat4::perspective_rh(to_rad(self.fov), aspect, self.znear, self.zfar);

        if current_matrix != self.perspective {
            self.updated = true;
        }
        self
    }

    pub fn get_near_clip(&self) -> f32 { self.znear }
    pub fn get_far_clip(&self) -> f32 { self.zfar }
    pub fn get_projection(&self) -> Mat4 { self.perspective }
    pub fn get_view(&self) -> Mat4 { self.view }
    pub fn get_data(&self) -> CameraUniformBuffer {
        CameraUniformBuffer {
            projection : self.perspective.to_cols_array_2d(),
            modelview : self.view.to_cols_array_2d()

            // projection : Mat4::IDENTITY.to_cols_array_2d(),
            // modelview : Mat4::IDENTITY.to_cols_array_2d()
        }
    }

    pub fn update_view_matrix(&mut self) {
        // let current_matrix = self.view;
        let mut rot_m = Mat4::IDENTITY;
        let trans_m = Mat4::from_translation(self.position);

        rot_m = rot_m * Mat4::from_axis_angle(Vec3::X, to_rad(self.rotation.x));
        rot_m = rot_m * Mat4::from_axis_angle(Vec3::Y, to_rad(self.rotation.y));
        rot_m = rot_m * Mat4::from_axis_angle(Vec3::Z, to_rad(self.rotation.z));

        self.view = trans_m * rot_m;
        self.updated = true;
    }

    // fn update(&mut self, delta : f32) {
    //     self.update = false;
    // }
}

