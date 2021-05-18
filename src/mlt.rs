use std::f64::consts::PI;

use crate::camera::{Camera, ImageBuffer};
use crate::scene::{Light, Object, Scene};
use crate::vector::Ray;
use crate::{CONTINUE_CHANCE, DISTANCE_FACTOR};
use nalgebra::Vector3;
use rand::{self, Rng};

pub fn draw(n: usize, x: f64, y: f64, light: &Light, scene: &Scene, image: &ImageBuffer) {
    let mut rng = rand::thread_rng();
    // // Choose a path by bidirectional path tracing
    let (p0, mut path) = scene.propose(x, y, light, &mut rng);
    let mut old_p = p0;
    for _ in 0..n {
        // these seem broken
        // let f = path.measure(scene);
        // let weight = f / p0;
        scene.camera.record_sample(
            &path,
            scene,
            image,
            10. / scene.lights.len() as f64 / n as f64,
        );
        if let Some((p, new_path)) = path.mutate(scene, &mut rng) {
            let measure = path.measure(scene);
            if measure == 0. {
                // accept unconditionally, old path was blocked
                // println!(".");
                old_p = p;
                path = new_path;
            } else {
                let accept = new_path.measure(scene) / path.measure(scene) * old_p / p;
                if rng.gen::<f64>() < accept * 10. {
                    // println!(".");
                    old_p = p;
                    path = new_path;
                } else {
                    // println!("new m {}", new_path.measure(scene));
                    // println!("m {}", path.measure(scene));
                    // println!("old p {}", old_p);
                    // println!("p {}", p);
                    // println!(" -> {}", accept);
                    // println!("-")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path<'a> {
    // order is from light
    pub light: &'a Light,
    pub objects: Vec<&'a Object>,
    pub camera: &'a Camera,
    // first element is point, second is normal at that point
    pub points: Vec<Vector3<f64>>,
    pub normals: Vec<Vector3<f64>>,
}

impl<'a> Path<'a> {
    // similar to camera work (should be deduplicated)
    fn measure(&self, scene: &Scene) -> f64 {
        // light pdf
        let mut prob = 1. / (4. * PI);
        for i in 0..self.objects.len() {
            let x0 = self.points[i];
            let x1 = self.points[i + 1];
            let x2 = self.points[i + 2];
            let incoming = x0 - x1;
            let outgoing = x2 - x1;
            let normal = self.normals[i];
            // check occlusion
            if let Some((t, _, _)) = scene.cast(Ray::new(x1, incoming)) {
                if t < 1. {
                    prob = 0.;
                    break;
                }
            }
            let mut geom = 1. / (1. + DISTANCE_FACTOR * incoming.magnitude_squared());
            geom *= incoming.normalize().dot(&normal);
            prob *= geom;

            // BSDF contribution
            let phi_in = incoming.angle(&normal);
            let phi_out = outgoing.angle(&normal);
            // project both onto the plane formed by the
            // this means that only symmetric distributions are allowed, due to the way it measureswe
            let proj_in = incoming - incoming.dot(&normal) / normal.magnitude_squared() * normal;
            let proj_out = outgoing - outgoing.dot(&normal) / normal.magnitude_squared() * normal;
            let theta = proj_in.angle(&proj_out);
            prob *= self.objects[i]
                .material
                .bsdf(phi_in, theta, phi_out)
                .luminance();
        }
        prob
    }
    fn mutate<R: Rng + ?Sized>(&self, scene: &'a Scene, rng: &mut R) -> Option<(f64, Path<'a>)> {
        match rng.gen_range(0..1) {
            // bidirectional mutation: regenerate part of the path
            0 => {
                let mut prob = 1.;
                if self.objects.len() > 0 {
                    let start = rng.gen_range(0..self.objects.len());
                    prob *= 1. / self.objects.len() as f64;
                    let end = rng.gen_range(start..self.objects.len());
                    prob *= 1. / (self.objects.len() - start) as f64;
                    let new_light_len = rng.gen_range(0..2);
                    prob *= 0.5;
                    let mut new_light = Vec::with_capacity(new_light_len);
                    let mut new_light_normals = Vec::with_capacity(new_light_len);
                    let mut new_light_objects = Vec::with_capacity(new_light_len);
                    for i in 0..new_light_len {
                        let (x0, normal, obj) = if i == 0 {
                            (
                                self.points[start + 1],
                                self.normals[start],
                                self.objects[start],
                            )
                        } else {
                            (
                                new_light[i - 1],
                                new_light_normals[i - 1],
                                new_light_objects[i - 1],
                            )
                        };
                        let (p, proposal) = obj.material.propose(normal, rng);
                        prob *= p;
                        let ray = Ray::new(x0, proposal);
                        if let Some((t, n, o)) = scene.cast(ray) {
                            new_light.push(ray.of(t));
                            new_light_normals.push(n);
                            new_light_objects.push(o)
                        } else {
                            return None;
                        }
                    }
                    let new_camera_len = rng.gen_range(0..2);
                    prob *= 0.5;

                    let mut new_camera = Vec::with_capacity(new_camera_len);
                    let mut new_camera_normals = Vec::with_capacity(new_camera_len);
                    let mut new_camera_objects = Vec::with_capacity(new_camera_len);
                    for i in 0..new_camera_len {
                        let (x0, normal, obj) = if i == 0 {
                            (self.points[end + 1], self.normals[end], self.objects[end])
                        } else {
                            (
                                new_camera[i - 1],
                                new_camera_normals[i - 1],
                                new_camera_objects[i - 1],
                            )
                        };
                        let (p, proposal) = obj.material.propose(normal, rng);
                        prob *= p;
                        let ray = Ray::new(x0, proposal);
                        if let Some((t, n, o)) = scene.cast(ray) {
                            new_camera.push(ray.of(t));
                            new_camera_normals.push(n);
                            new_camera_objects.push(o)
                        } else {
                            return None;
                        }
                    }
                    let mut points = self.points[..=start + 1].to_owned();
                    points.extend(new_light);
                    points.extend(new_camera.into_iter().rev());
                    points.extend(self.points[end + 1..].iter().map(|p| *p));
                    let mut normals = self.normals[..=start].to_owned();
                    normals.extend(new_light_normals);
                    normals.extend(new_camera_normals.into_iter().rev());
                    normals.extend(self.normals[end..].iter().map(|p| *p));
                    let mut objects = self.objects[..=start].to_owned();
                    objects.extend(new_light_objects);
                    objects.extend(new_camera_objects.into_iter().rev());
                    objects.extend(self.objects[end..].iter().map(|p| *p));
                    return Some((
                        prob,
                        Path {
                            light: self.light,
                            objects,
                            camera: self.camera,
                            points,
                            normals,
                        },
                    ));
                }
            }
            _ => unreachable!(),
        }
        None
    }
}

impl Scene {
    /// All the propose methods give a path (or ray) and the probability of generating it
    pub fn propose<'a, R: Rng + ?Sized>(
        &'a self,
        x: f64,
        y: f64,
        light: &'a Light,
        rng: &mut R,
    ) -> (f64, Path) {
        // let light = self.lights.choose(rng).unwrap();
        // this term gets cancelled out anyway
        let mut prob = 1.; // self.lights.len() as f64;
        let camera = &self.camera;
        let mut light_points = vec![light.pos];
        let mut light_objects = vec![];
        let mut light_normals = vec![];
        let mut camera_points = vec![camera.pos];
        let mut camera_objects = vec![];
        let mut camera_normals = vec![];
        // cast camera ray
        let r = camera.propose(x, y);
        // prob *= p;
        let ray = Ray::new(camera.pos, r);
        if let Some((t, n, o)) = self.cast(ray) {
            camera_points.push(ray.of(t));
            camera_normals.push(n);
            camera_objects.push(o);

            if rng.gen_bool(CONTINUE_CHANCE) {
                prob *= CONTINUE_CHANCE;
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
                        prob *= CONTINUE_CHANCE;

                        // add a new camera point
                        let (p, r) = camera_objects
                            .last()
                            .unwrap()
                            .material
                            .propose(*camera_normals.last().unwrap(), rng);
                        prob *= p;
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
                        prob *= CONTINUE_CHANCE;

                        // add a new light point
                        let (p, r) = light_objects
                            .last()
                            .unwrap()
                            .material
                            .propose(*light_normals.last().unwrap(), rng);
                        prob *= p;
                        let ray = Ray::new(*light_points.last().unwrap(), r);
                        if let Some((t, n, o)) = self.cast(ray) {
                            light_points.push(ray.of(t));
                            light_normals.push(n);
                            light_objects.push(o);
                        } else {
                            break;
                        }
                    }
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
