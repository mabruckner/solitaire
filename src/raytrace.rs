use std::f32;
use cgmath;
use cgmath::{
    Matrix4,
    SquareMatrix,
    Vector4,
};
use amethyst;
use amethyst::renderer::Camera;



pub fn res_cam_to_ren(cam: &amethyst::ecs::resources::Camera) -> Camera {
    let view = Camera::look_at(cam.eye, cam.target, cam.up);
    let proj = match cam.proj {
        amethyst::ecs::resources::Projection::Orthographic{left,right,top,bottom,near,far} => {
            Camera::orthographic(left,right,top,bottom, near,far)
        },
        amethyst::ecs::resources::Projection::Perspective{fov, aspect_ratio, near, far} => {
            Camera::perspective(fov, aspect_ratio, near, far)
        }
    };
    Camera::new(proj, view)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ray {
    pub start: [f32; 3],
    pub velocity: [f32; 3],
}

impl Ray {
    pub fn reverse_transform(&self, mat: [[f32; 4]; 4]) -> Self {
        let mat = Matrix4::from(mat);
        let mat = mat.invert().unwrap();
        Ray {
            start: (mat * Vector4::from([self.start[0], self.start[1], self.start[2], 1.0])).truncate().into(),
            velocity: (mat * Vector4::from([self.velocity[0], self.velocity[1], self.velocity[2], 0.0])).truncate().into()
        }
    }
    pub fn from_camera_mouse(cam: Camera, screen: (f32, f32), mouse: (i32, i32)) -> Self {
        let rel = (2.0*(mouse.0 as f32)/screen.0-1.0, 2.0*(mouse.1 as f32)/screen.1-1.0);
        let base = Ray {
            start:[rel.0, rel.1, 0.0],
            velocity:[0.0, 0.0, 1.0]
        };
        let base = base.reverse_transform(cam.proj);
        let base = base.reverse_transform(cam.view);
        base
    }
    pub fn along(&self, val: f32) -> [f32; 3] {
        let mut out = [0.0;3];
        for i in 0..3 {
            out[i] = self.start[i]+val*self.velocity[i];
        }
        out
    }
}

pub trait Raytraceable {
    fn raytrace(&self, ray: &Ray) -> Option<f32>;
}

pub struct Box {
    dims: [f32; 3]
}

impl Box {
    pub fn new(w:f32,h:f32,d:f32) -> Box {
        Box{ dims: [w,h,d] }
    }
}

impl Raytraceable for Box {
    fn raytrace(&self, ray: &Ray) -> Option<f32> {
        let mut results = [(0.0, 0.0); 3];
        for i in 0..3 {
            results[i] = if ray.velocity[i] == 0.0 {
                if ray.start[i].abs() <= self.dims[i]/2.0 {
                    (f32::NEG_INFINITY, f32::INFINITY)
                } else {
                    return None;
                }
            } else {
                let offset = -ray.start[i]/ray.velocity[i];
                let span = (self.dims[i]/(2.0*ray.velocity[i])).abs();
                (offset - span, offset + span)
            }
        }
        println!("here? {:?}", ray);
        let mut range = (f32::NEG_INFINITY, f32::INFINITY);
        for i in 0..3 {
            range.0 = results[i].0.max(range.0);
            range.1 = results[i].1.min(range.1);
        }
        println!("range:{:?}", range);
        if range.0 > range.1 {
            None
        } else {
            Some(range.0)
        }
    }
}
