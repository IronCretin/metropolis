use std::thread::sleep;
use std::time::Duration;

use crate::camera::{Camera, ImageBuffer};
use crate::scene::{Light, Object, Scene};
use crate::vector::Ray;
use crate::CONTINUE_CHANCE;
use nalgebra::Vector3;
use rand::prelude::SliceRandom;
use rand::{self, Rng};

#[derive(Debug, Clone)]
pub struct Path<'a> {
    // order is from camera
    pub camera: &'a Camera,
    pub objects: Vec<&'a Object>,
    pub light: &'a Light,
    // first element is point, second is normal at that point
    pub points: Vec<Vector3<f64>>,
    pub normals: Vec<Vector3<f64>>,
}

pub fn draw(n: usize, scene: &Scene, image: ImageBuffer) {
    let mut rng = rand::thread_rng();
    // // Choose a path by bidirectional path tracing
    for _ in 0..n {
        let (p0, mut path) = scene.propose(&mut rng);
        scene.camera.record_sample(&path, scene, image);
        // let new_path = path.mutate(scene, &mut rng);
        // if rng.gen::<f64>() < new_path.accept_probability(path) {
        //     path = new_path;
        // }
    }
    // for i in 0..image.width {
    //     for j in 0..image.width {
    //         image.buffer.lock().unwrap()[i * image.width + j] = 0xff_ff_ff_ff;
    //     }
    // }
}

impl Scene {
    /// All the propose methods give a path (or ray) and the probability of generating it
    pub fn propose<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, Path) {
        let light = self.lights.choose(rng).unwrap();
        let mut prob = 1. / self.lights.len() as f64;
        let camera = &self.camera;
        let mut light_points = vec![light.pos];
        let mut light_objects = vec![];
        let mut light_normals = vec![];
        let mut camera_points = vec![camera.pos];
        let mut camera_objects = vec![];
        let mut camera_normals = vec![];
        // cast camera ray
        loop {
            let (p, r) = camera.propose(rng);
            prob *= p;
            let ray = Ray::new(camera.pos, r);
            if let Some((t, n, o)) = self.cast(ray) {
                camera_points.push(ray.of(t));
                camera_normals.push(n);
                camera_objects.push(o);
                break;
            }
        }
        if rng.gen_bool(CONTINUE_CHANCE) {
            // cast a ray from the light
            let (p, r) = light.propose(rng);
            prob *= p;
            let ray = Ray::new(light.pos, r);
            if let Some((t, n, o)) = self.cast(ray) {
                light_points.push(ray.of(t));
                light_normals.push(n);
                light_objects.push(o);
                loop {
                    if !rng.gen_bool(CONTINUE_CHANCE) {
                        break;
                    }
                    // add a new camera point
                    let (p, r) = camera_objects
                        .last()
                        .unwrap()
                        .material
                        .propose(*camera_normals.last().unwrap(), rng);
                    let ray = Ray::new(*camera_points.last().unwrap(), r);
                    if let Some((t, n, o)) = self.cast(ray) {
                        camera_points.push(ray.of(t));
                        camera_normals.push(n);
                        camera_objects.push(o);
                    } else {
                        break;
                    }

                    if !rng.gen_bool(CONTINUE_CHANCE) {
                        break;
                    }
                    let (p, r) = light_objects
                        .last()
                        .unwrap()
                        .material
                        .propose(*light_normals.last().unwrap(), rng);
                    let ray = Ray::new(*light_points.last().unwrap(), r);
                    if let Some((t, n, o)) = self.cast(ray) {
                        light_points.push(ray.of(t));
                        light_normals.push(n);
                        light_objects.push(o);
                    } else {
                        break;
                    }

                    // add a new light point
                }
            }
        }
        light_points.extend(camera_points.iter().rev());
        light_objects.extend(camera_objects.iter().rev());
        light_normals.extend(camera_normals.iter().rev());
        (
            prob,
            Path {
                light,
                objects: light_objects,
                camera,
                points: light_points,
                normals: light_normals,
            },
        )
    }
}
