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

use crossbeam_utils::thread;
use minifb::{Window, WindowOptions};
use nalgebra::Vector3;

use crate::camera::{Camera, ImageBuffer};
use crate::color::Color;
use crate::material::Material;
use crate::mlt::draw;
use crate::scene::{Light, Object, Scene, Shape};

const WIDTH: usize = 640;
const HEIGHT: usize = 640;
const N_THREADS: usize = 1;

// do not consider intersections closer than this. (prevents shadow acne)
const MIN_DIST: f64 = 0.0001;

// chance of adding another step to the traced path
const CONTINUE_CHANCE: f64 = 0.5;

// factor for light attenuation over distance
const DISTANCE_FACTOR: f64 = 0.1;

fn main() {
    let buffer = Mutex::new(vec![Color::new(0., 0., 0.); WIDTH * HEIGHT]);
    let image = ImageBuffer {
        buffer: &buffer,
        width: WIDTH,
        height: HEIGHT,
    };

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

    let scene = Scene {
        camera: Camera::new(
            Vector3::new(0., 0., -4.),
            Vector3::new(0., 0., 1.),
            Vector3::new(0., 1., 0.),
            PI / 5.,
        ),
        lights: vec![Light {
            pos: Vector3::new(2., 2., -2.),
            color: Color::new(1., 1., 1.),
        }],
        objects: vec![
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(0., 0., 0.),
                    radius: 1.,
                },
                material: Material::Diffuse(Color::new(1., 0.5, 0.5)),
            },
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(-1., -1., -1.),
                    radius: 0.5,
                },
                material: Material::Diffuse(Color::new(0.5, 1., 0.5)),
            },
            Object {
                shape: Shape::Sphere {
                    center: Vector3::new(1., 1., -1.),
                    radius: 0.2,
                },
                material: Material::Diffuse(Color::new(0.5, 0.5, 1.)),
            },
            // Object {
            //     shape: Shape::Sphere {
            //         center: Vector3::new(0., 0., 0.),
            //         radius: 4.,
            //     },
            //     material: Material::Diffuse(Color::new(1., 1., 1.)),
            // },
            // Object {
            //     shape: Shape::Plane {
            //         center: Vector3::new(0., -1., 0.),
            //         normal: Vector3::new(0., 1., 0.),
            //     },
            //     material: Material::Diffuse(Color::new(0., 1., 0.)),
            // },
        ],
    };

    thread::scope(|scope| {
        for i in 0..N_THREADS {
            scope.spawn(|_| draw(1_000_000, &scene, image));
        }
        let mut buffer = vec![0u32; WIDTH * HEIGHT];
        while window.is_open() {
            if let Ok(colors) = image.buffer.lock() {
                for (i, c) in colors.iter().enumerate() {
                    buffer[i] = (*c).into();
                }
            }

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window
                .update_with_buffer(&buffer, image.width, image.height)
                .unwrap();
            // Sleep for a frame to let the renderer do it's work (this makes sure we aren't holding the buffer mutex open)
            sleep(Duration::from_micros(166000));
        }
        exit(0)
    })
    .unwrap();
}
