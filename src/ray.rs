use crate::vec3::{Point3, Vec3};

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub(crate) fn new(orig: Point3, dir: Vec3) -> Self {
        Self { orig, dir }
    }

    #[allow(dead_code)]
    pub(crate) fn origin(&self) -> Point3 {
        self.orig
    }

    pub(crate) fn direction(&self) -> Vec3 {
        self.dir
    }

    #[allow(dead_code)]
    pub(crate) fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }
}
