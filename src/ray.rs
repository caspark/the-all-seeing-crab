use crate::vec3::{Point3, Vec3};

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
    pub tm: f64,
}

impl Ray {
    pub(crate) fn new(orig: Point3, dir: Vec3, time: Option<f64>) -> Self {
        Self {
            orig,
            dir,
            tm: time.unwrap_or(0.0),
        }
    }

    pub(crate) fn origin(&self) -> Point3 {
        self.orig
    }

    pub(crate) fn direction(&self) -> Vec3 {
        self.dir
    }

    pub(crate) fn time(&self) -> f64 {
        self.tm
    }

    pub(crate) fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }
}
