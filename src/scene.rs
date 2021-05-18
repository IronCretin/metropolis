use std::f64::consts::{PI, TAU};
use std::f64::INFINITY;
use std::fmt::Debug;

use nalgebra::Vector3;
use rand::Rng;

use crate::camera::Camera;
use crate::color::Color;
use crate::material::Material;
use crate::vector::Ray;
use crate::MIN_DIST;

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn cast(&self, ray: Ray<f64>) -> Option<(f64, Vector3<f64>, &Object)> {
        let mut intersection = None;
        let mut min_dist = INFINITY;
        for obj in &self.objects {
            if let Some((t, norm)) = obj.shape.cast(ray) {
                if t < min_dist {
                    min_dist = t;
                    intersection = Some((t, norm, obj))
                }
            }
        }
        intersection
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub material: Material,
    pub shape: Shape,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Sphere {
        center: Vector3<f64>,
        radius: f64,
    },
    Plane {
        center: Vector3<f64>,
        normal: Vector3<f64>,
    },
}

impl Shape {
    pub fn cast(&self, ray: Ray<f64>) -> Option<(f64, Vector3<f64>)> {
        let dir = ray.dir;
        match self {
            Shape::Sphere { center, radius } => {
                let a = dir.norm_squared();
                let b = 2. * dir.dot(&(ray.start - center));
                let c = (ray.start - center).norm_squared() - radius * radius;
                let discr = b * b - 4. * a * c;
                if discr >= 0. {
                    let t = (-b - discr.sqrt()) / (2. * a);
                    if t > MIN_DIST {
                        return Some((t, (ray.of(t) - center).normalize()));
                    }
                    let t = (-b + discr.sqrt()) / (2. * a);
                    if t > MIN_DIST {
                        // we are inside the sphere
                        return Some((t, center - ray.of(t)));
                    }
                }
            }
            Shape::Plane { center, normal } => {
                let t = normal.dot(&(center - ray.start)) / normal.dot(&ray.dir);
                if t > MIN_DIST {
                    return Some((t, *normal));
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    pub pos: Vector3<f64>,
    pub color: Color,
}

impl Light {
    pub fn propose<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, Vector3<f64>) {
        let theta = rng.gen_range(0. ..TAU);
        // Archimedes' hat-box theorem lets us generate a z-value and convert it to an angle
        let z: f64 = rng.gen_range(-1. ..1.);
        let r = (1. - z * z).sqrt();
        (
            1. / (4. * PI),
            Vector3::new(theta.cos() * r, theta.sin() * r, z),
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn casting() {
        let scene = Scene {
            camera: Camera::new(
                Vector3::new(0., 0., 2.),
                -Vector3::z(),
                Vector3::y(),
                PI / 4.,
            ),
            lights: vec![],
            objects: vec![Object {
                shape: Shape::Sphere {
                    center: Vector3::new(0., 0., 0.),
                    radius: 1.,
                },
                material: Material::Diffuse(Color::new(1., 1., 1.)),
            }],
        };
        let rng = &mut rand::thread_rng();
        for i in 0..100 {
            let (_, dir) = scene.camera.propose(rng);
            let ray = Ray::new(scene.camera.pos, dir);
            if let Some((t, _, _)) = scene.cast(ray) {
                let x = ray.of(t);
                assert_abs_diff_eq!(x.norm(), 1.);
                assert!(x[2] > 0.);
            }
        }
    }
}
