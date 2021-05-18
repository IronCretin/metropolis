use std::f64::consts::{PI, TAU};
use std::fmt::Debug;
use std::ops::Add;

use nalgebra::{Rotation3, Vector3};
use rand::Rng;

use crate::color::Color;

#[derive(Debug, Clone)]
pub enum Material {
    Diffuse(Color),
    Specular(Color, f64),
    Combined(Vec<(f64, Material)>),
}

impl Material {
    /// Bidirectional scattering distribution function -- the probability that a
    /// light ray incoming at phi_in will be reflected in the direction angled
    /// by phi_out and theta (theta is measured from the incoming source)
    /// Measured in three color channels.
    /// phi is the angle from the normal axis.
    pub fn bsdf(&self, phi_in: f64, theta: f64, phi_out: f64) -> Color {
        match self {
            // we don't need to weight by the sine because points are generated
            // uniformly on the plane, just converted to angles for convenience
            &Self::Diffuse(color) => color, //* phi_out.sin(),
            &Self::Specular(color, alpha) => {
                // dot product of outgoing vector with reflection, raised to exponent
                let dot =
                    -theta.cos() * phi_in.sin() * phi_out.sin() + phi_in.cos() * phi_out.cos();
                color * dot.abs().powf(alpha)
            }
            Self::Combined(mats) => mats
                .iter()
                .map(|(w, m)| m.bsdf(phi_in, theta, phi_out) * *w)
                .sum(),
        }
    }
    fn color(&self) -> Color {
        match self {
            &Self::Diffuse(color) => color,
            &Self::Specular(color, _) => color,
            Self::Combined(mats) => mats.iter().map(|(w, m)| m.color() * *w).sum(),
        }
    }
    /// By default generate a random point on the hemisphere
    pub fn propose<R: Rng + ?Sized>(
        &self,
        normal: Vector3<f64>,
        rng: &mut R,
    ) -> (f64, Vector3<f64>) {
        let theta = rng.gen_range(0. ..TAU);
        // Archimedes' hat-box theorem lets us generate a z-value and convert it to an angle
        let z: f64 = rng.gen_range(0. ..1.);
        let r = (1. - z * z).sqrt();

        (
            self.color().luminance() / (2. * PI),
            Rotation3::rotation_between(&Vector3::z(), &normal).unwrap_or(Rotation3::identity())
                * Vector3::new(theta.cos() * r, theta.sin() * r, z),
        )
    }
}
