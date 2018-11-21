
use std::f32;
use nalgebra::{Isometry3, Perspective3, Point3, Vector3, Matrix4};

pub(super) struct Camera {
    view: Isometry3<f32>,
    projection: Perspective3<f32>,
}

impl Camera {
    pub fn new(aspect : f32, fov_deg : f32, eye: Point3<f32>, target: Point3<f32>) -> Camera {

        let projection = Perspective3::new(aspect, deg_to_rad(fov_deg), 0.01, 100.0);

        let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());

        Camera {
            view,
            projection,
        }
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        self.view.to_homogeneous()
    }

    pub fn get_projection(&self) -> Matrix4<f32> {
        self.projection.to_homogeneous()
    }
}


fn deg_to_rad(deg: f32) -> f32 {
    deg * f32::consts::PI / 180.0
}