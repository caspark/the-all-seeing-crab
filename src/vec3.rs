use std::ops::{Div, DivAssign, Index, IndexMut, Mul, MulAssign};

use derive_more::{Add, AddAssign, Constructor, Display, Sub, SubAssign, Sum};

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Constructor,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Display,
)]
#[display(fmt = "{} {} {}", x, y, z)]
struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    fn zero() -> Self {
        Default::default()
    }

    fn one() -> Self {
        Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn to_unit(&self) -> Vec3 {
        *self / self.length()
    }
}

impl Into<Vec3> for [f64; 3] {
    fn into(self) -> Vec3 {
        Vec3::new(self[0], self[1], self[2])
    }
}

impl Mul<Self> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self // rely on existing trait impl: Mul<f64> for Vec3
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1.0 / rhs;
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index for indexing vec3 {}", index),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index for mutably indexing vec3 {}", index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(Vec3::default(), Vec3::zero());
    }

    #[test]
    fn new() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn length() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.length_squared(), 1.0 + 4.0 + 9.0);
        assert_eq!(v.length(), (1.0f64 + 4.0 + 9.0).sqrt());
    }

    #[test]
    fn dot() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.dot(b), 4.0 + 10.0 + 18.0);
    }

    #[test]
    fn cross() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.cross(b), Vec3::new(-3.0, 6.0, -3.0));
    }

    #[test]
    fn to_unit() {
        let a = Vec3::new(1.0, 2.0, 2.0);
        assert_eq!(a.to_unit(), Vec3::new(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    }

    #[test]
    fn add() {
        assert_eq!(Vec3::one() + Vec3::one(), Vec3::new(2.0, 2.0, 2.0));

        let mut v = Vec3::one();
        v += Vec3::one();
        assert_eq!(v, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn sub() {
        let two = Vec3::new(2.0, 2.0, 2.0);
        let three = Vec3::new(3.0, 3.0, 3.0);
        assert_eq!(three - Vec3::one(), two);

        let mut v = three.clone();
        v -= Vec3::one();
        assert_eq!(v, two);
    }

    #[test]
    fn mul_vec() {
        let initial = Vec3::new(1.0, 2.0, 3.0);
        let multiplier = Vec3::new(2.0, 4.0, 6.0);
        let result = Vec3::new(2.0, 8.0, 18.0);
        assert_eq!(initial * multiplier, result);
        assert_eq!(multiplier * initial, result);
    }

    #[test]
    fn mul_scalar() {
        let initial = Vec3::new(1.0, 2.0, 3.0);
        let tripled = Vec3::new(3.0, 6.0, 9.0);
        assert_eq!(initial * 3.0, tripled);
        assert_eq!(3.0 * initial, tripled);

        let mut v = initial.clone();
        v *= 3.0;
        assert_eq!(v, tripled);
    }

    #[test]
    fn div() {
        let initial = Vec3::new(3.0, 6.0, 9.0);
        let thirded = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(initial / 3.0, thirded);

        let mut v = initial.clone();
        v /= 3.0;
        assert_eq!(v, thirded);
    }

    #[test]
    fn index() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);
        assert_eq!(v[2], 3.0);
    }

    #[test]
    fn indexmut() {
        let mut v = Vec3::default();
        v[0] = 1.0;
        v[1] = 2.0;
        v[2] = 3.0;

        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn display() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(format!("{}", v), "1 2 3");
    }
}
