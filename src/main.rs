mod camera;
mod color;
mod material;
mod mlt;
mod scene;
mod vector;

use std::f64::consts::PI;
use std::process::exit;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use std::usize;

use lazy_static::lazy_static;
use minifb::{Window, WindowOptions};
use nalgebra::Vector3;
use rayon::ThreadPoolBuilder;

use crate::camera::{Camera, ImageBuffer};
use crate::color::Color;
use crate::material::Material;
use crate::mlt::{draw, Path};
use crate::scene::{Light, Object, Scene, Shape};

const WIDTH: usize = 640;
const HEIGHT: usize = 640;
const N_THREADS: usize = 8;

const SAMPLES_PER_PIXEL: usize = 50;

// do not consider intersections closer than this. (mostly prevents shadow acne)
const MIN_DIST: f64 = 0.001;

// chance of adding another step to the traced path
const CONTINUE_CHANCE: f64 = 0.5;

// factor for light attenuation over distance
const DISTANCE_FACTOR: f64 = 0.1;

lazy_static! {
    static ref IMAGE: ImageBuffer = ImageBuffer {
        buffer: Mutex::new(vec![Color::new(0., 0., 0.); WIDTH * HEIGHT]),
        width: WIDTH,
        height: HEIGHT,
    };
    static ref SCENE: Scene = Scene {
        camera: Camera::new(
            Vector3::new(0., 0., -4.),
            Vector3::new(0., 0., 1.),
            Vector3::new(0., 1., 0.),
            PI / 5.,
        ),
        lights: vec![
            // Light {
            //     pos: Vector3::new(-1.5, 1.5, -1.5),
            //     color: Color::new(1., 0., 0.),
            // },
            // Light {
            //     pos: Vector3::new(0., 1.5, -1.5),
            //     color: Color::new(0., 0., 1.),
            // },
            Light {
                pos: Vector3::new(1.5, 1.5, -1.5),
                color: Color::new(1., 1., 1.),
            },
        ],
        objects: vec![
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(0., 0., 0.),
                    radius: 1.,
                },
                material: Material::Combined(vec![
                    (0.6, Material::Diffuse(Color::new(1., 0.5, 0.5))),
                    (0.4, Material::Specular(Color::new(1., 0.5, 0.5), 10.)),
                    ]),
            },
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(-1., -1., -1.),
                    radius: 0.5,
                },
                material:Material::Diffuse(Color::new(0.5, 1., 0.5)),
            },
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(1., 1., -1.),
                    radius: 0.2,
                },
                material: Material::Specular(Color::new(1., 1., 1.), 10.),
            },
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(0., 0., 0.),
                    radius: 4.,
                },
                material: Material::Diffuse(Color::new(0., 0., 0.5)),
            },
            Object {
                shape: Shape::Plane {
                    center: Vector3::new(0., -2., 0.),
                    normal: Vector3::new(0., 1., 0.),
                },
                material: Material::Combined(vec![
                    (0.2, Material::Diffuse(Color::new(1., 1., 1.))),
                    (0.8, Material::Specular(Color::new(1., 1., 1.), 5.)),
                    ]),
            },
        ],
    };
}

fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(Duration::from_micros(16600)));

    let pool = ThreadPoolBuilder::new()
        .num_threads(N_THREADS)
        .build()
        .unwrap();
    // draw lights
    for light in &SCENE.lights {
        SCENE.camera.record_sample(
            &Path {
                camera: &SCENE.camera,
                light,
                normals: vec![],
                objects: vec![],
                points: vec![light.pos, SCENE.camera.pos],
            },
            &SCENE,
            &IMAGE,
            1.,
        )
    }

    println!("spawning threads...");
    for i in 0..WIDTH {
        let x = 2. * (i as i32 - WIDTH as i32 / 2) as f64 / WIDTH as f64;
        for j in 0..HEIGHT {
            let y = 2. * (j as i32 - HEIGHT as i32 / 2) as f64 / WIDTH as f64;
            // do this to account for multiple lights
            for light in &SCENE.lights {
                pool.spawn(move || {
                    draw(
                        SAMPLES_PER_PIXEL / SCENE.lights.len(),
                        x,
                        y,
                        light,
                        &SCENE,
                        &IMAGE,
                    )
                });
            }
        }
    }
    println!("rendering...");
    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    while window.is_open() {
        if let Ok(colors) = IMAGE.buffer.try_lock() {
            for (i, c) in colors.iter().enumerate() {
                buffer[i] = (*c).into();
            }
        }

        // Unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, IMAGE.width, IMAGE.height)
            .unwrap();
        // Sleep for a frame to let the renderer do it's work (this makes sure we aren't holding the buffer mutex open)
        sleep(Duration::from_micros(16600));
    }

    exit(0)
}
