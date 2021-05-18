use std::sync::Mutex;
use std::usize;

use nalgebra::{Rotation3, Vector3};
use rand::Rng;

use crate::color::Color;
use crate::material::Material;
use crate::mlt::Path;
use crate::scene::Scene;
use crate::vector::Ray;
use crate::DISTANCE_FACTOR;

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vector3<f64>,
    // currently ignore these
    // pub facing: Vector3<f64>,
    // pub up: Vector3<f64>,
    rotation: Rotation3<f64>,
    // focal length
    f: f64,
    /// Angular distance to each edge of the lens
    pub fov: f64,
}

impl Camera {
    pub fn new(
        pos: Vector3<f64>,
        mut facing: Vector3<f64>,
        mut up: Vector3<f64>,
        fov: f64,
    ) -> Self {
        facing.normalize_mut();
        up.normalize_mut();
        Camera {
            rotation: Rotation3::look_at_lh(&facing, &up),
            f: 1. / fov.tan(),
            pos,
            fov,
        }
    }
    pub fn record_sample(&self, path: &Path, scene: &Scene, image: ImageBuffer) {
        let point = self.rotation.transform_vector(
            &(path.points[path.points.len() - 2] - path.points[path.points.len() - 1]),
        );
        let projected = self.f / point[2] * point;
        if -1. <= projected[0] && projected[0] < 1. && -1. <= projected[1] && projected[1] < 1. {
            let x = ((projected[0] + 1.) * image.height as f64 / 2.) as usize;
            let y = ((-projected[1] + 1.) * image.height as f64 / 2.) as usize;
            let mut color = path.light.color / (1. + DISTANCE_FACTOR * point.magnitude_squared());

            for i in 0..path.objects.len() {
                let x0 = path.points[i];
                let x1 = path.points[i + 1];
                let x2 = path.points[i + 2];
                let incoming = x0 - x1;
                let outgoing = x2 - x1;
                let normal = path.normals[i];
                // check occlusion
                if let Some((t, _, _)) = scene.cast(Ray::new(x1, incoming)) {
                    if t < 1. {
                        color *= 0.;
                        break;
                    }
                }
                let mut geom = 1. / (1. + DISTANCE_FACTOR * incoming.magnitude_squared());
                geom *= incoming.normalize().dot(&normal);
                color *= geom;

                // BSDF contribution
                let phi_in = incoming.angle(&normal);
                let phi_out = outgoing.angle(&normal);
                // project both onto the plane formed by the
                // this means that only symmetric distributions are allowed, due to the way we measure
                let proj_in =
                    incoming - incoming.dot(&normal) / normal.magnitude_squared() * normal;
                let proj_out =
                    outgoing - outgoing.dot(&normal) / normal.magnitude_squared() * normal;
                let theta = proj_in.angle(&proj_out);
                color *= path.objects[i].material.bsdf(phi_in, theta, phi_out)

                // color *= path.objects[i]
            }
            // color = color
            //     * (path.points[0] - path.points[1])
            //         .normalize()
            //         .dot(&(path.points[2] - path.points[1]).normalize());
            // println!(
            //     "{}",
            //     (path.points[0] - path.points[1])
            //         .normalize()
            //         .dot(&(path.points[2] - path.points[1]))
            // );
            image.buffer.lock().unwrap()[image.width * y + x] += color;
        }
    }
    pub fn propose<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, Vector3<f64>) {
        let x = rng.gen_range(-1. ..1.);
        let y = rng.gen_range(-1. ..1.);
        let v = Vector3::new(x, y, self.f);
        (1. / 4., self.rotation.inverse_transform_vector(&v))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ImageBuffer<'a> {
    pub buffer: &'a Mutex<Vec<Color>>,
    pub width: usize,
    pub height: usize,
}
