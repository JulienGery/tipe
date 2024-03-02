use cgmath::{Matrix4, Rad, SquareMatrix};
use cgmath::{Vector4, Vector3};

pub struct Camera {
    position : Vector3<f32>,
    rotation : Vector3<f32>,
    view_pos : Vector4<f32>,
    perspective : Matrix4<f32>,
    view : Matrix4<f32>,
    fov : f32,
    znear : f32,
    zfar : f32,
    update : bool
}

fn to_rad(theta : f32) -> Rad<f32> {
    cgmath::Rad::from(cgmath::Deg{0: theta})
}

impl Camera {
    pub fn new() -> Self {
        let position = Vector3::new(0., 0., 0.);
        let rotation = position.clone();
        let view_pos = Vector4::new(0., 0., 0., 1.);
        let perspective = Matrix4::new(0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.);
        let view = Matrix4::identity();

        Self {
            position,
            rotation,
            view_pos,
            perspective,
            update : true,
            fov : 20.,
            znear : 0.,
            zfar : 1.,
            view
        }
    }

    pub fn set_position(&mut self, position : Vector3<f32>) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation : Vector3<f32>) {
        self.rotation = rotation;
    }

    pub fn set_perspective(&mut self, fov : f32, aspect : f32, znear : f32, zfar : f32) {
        let current_matrix = self.perspective;
        self.fov = fov;
        self.znear = znear;
        self.zfar = zfar;
        self.perspective = cgmath::perspective(to_rad(fov), aspect, znear, zfar);

        if current_matrix != self.perspective {
            self.update = true;

        }
    }

    pub fn get_near_clip(&self) -> f32 { self.znear }
    pub fn get_far_clip(&self) -> f32 { self.zfar }

    fn update_view_matrix(&mut self) {
        let current_matrix = self.view;
        let mut rot_m : Matrix4<f32> = Matrix4::identity();
        let trans_m = Matrix4::from_translation(self.position);

        rot_m = rot_m * Matrix4::from_axis_angle(Vector3::unit_x(), to_rad(self.rotation.x));
        rot_m = rot_m * Matrix4::from_axis_angle(Vector3::unit_y(), to_rad(self.rotation.y));
        rot_m = rot_m * Matrix4::from_axis_angle(Vector3::unit_z(), to_rad(self.rotation.z));

        self.view = trans_m * rot_m;
        self.update = true;
    }

    // fn update(&mut self, delta : f32) {
    //     self.update = false;
    // }
}
