use std::ops::Neg;
use std::iter::IntoIterator;
use std::f32;

#[derive(Debug, Default, Clone)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3f {
        Vector3f { x, y, z}
    }

    pub fn length(&self) -> f32 {
        let sq_sum = self.x * self.x + self.y * self.y + self.z * self.z;
        sq_sum.sqrt()
    }
}

impl Neg for Vector3f {
    type Output = Vector3f;

    fn neg(mut self) -> Vector3f {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl<'a> Neg for &'a Vector3f {
    type Output = Vector3f;

    fn neg(self) -> Vector3f {
        Vector3f {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl IntoIterator for Vector3f {
    type Item = f32;
    type IntoIter = ::std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.x, self.y, self.z].into_iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

impl Vector2f {
    pub fn new(x: f32, y: f32) -> Vector2f {
        Vector2f { x, y, }
    }
    pub fn length(&self) -> f32 {
        let sq_sum = self.x * self.x + self.y * self.y;
        sq_sum.sqrt()
    }
}

impl IntoIterator for Vector2f {
    type Item = f32;
    type IntoIter = ::std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.x, self.y].into_iter()
    }
}