use nalgebra::{SimdRealField, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Ray<T> {
    pub start: Vector3<T>,
    pub dir: Vector3<T>,
}
impl<T> Ray<T> {
    pub const fn new(start: Vector3<T>, dir: Vector3<T>) -> Self {
        Ray { start, dir }
    }
    pub fn of(&self, t: T) -> Vector3<T>
    where
        T: SimdRealField,
    {
        self.start + self.dir * t
    }
}
