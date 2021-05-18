use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b }
    }
}
impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        fn map(c: f64) -> u32 {
            // clamp the color to the 0..255 range
            if c >= 1. {
                255
            } else if c < 0. {
                0
            } else {
                (c * 256.).trunc() as u32
            }
        }
        map(color.r) << 16 | map(color.g) << 8 | map(color.b)
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}
impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}
impl Sum for Color {
    fn sum<I: Iterator<Item = Color>>(iter: I) -> Self {
        iter.fold(Color::new(0., 0., 0.), Color::add)
    }
}

impl Mul for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b + rhs.b,
        }
    }
}
impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}

impl Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Color {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}
impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}
impl Div<f64> for Color {
    type Output = Color;
    fn div(self, rhs: f64) -> Color {
        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_into() {
        let c1 = u32::from(Color::new(1., 0.5, 0.));
        let c2 = 0x00_ff_80_00;
        assert_eq!(c1, c2, "{:x} != {:x}", c1, c2);
    }
}
