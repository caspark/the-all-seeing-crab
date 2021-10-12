use crate::{ray::Ray, vec3::Point3};

use derive_more::Constructor;

#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct Aabb {
    pub minimum: Point3,
    pub maximum: Point3,
}

impl Aabb {
    pub(crate) fn min(&self) -> Point3 {
        self.minimum
    }

    pub(crate) fn max(&self) -> Point3 {
        self.maximum
    }

    pub(crate) fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> bool {
        for a in 0..3 {
            let t0 = f64::min(
                (self.minimum[a] - r.origin()[a]) / r.direction()[a],
                (self.maximum[a] - r.origin()[a]) / r.direction()[a],
            );
            let t1 = f64::max(
                (self.minimum[a] - r.origin()[a]) / r.direction()[a],
                (self.maximum[a] - r.origin()[a]) / r.direction()[a],
            );
            let t_min = f64::max(t0, t_min);
            let t_max = f64::min(t1, t_max);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub(crate) fn surrounding_box(box0: Aabb, box1: Aabb) -> Aabb {
        let small = Point3::new(
            f64::min(box0.min().x, box1.min().x),
            f64::min(box0.min().y, box1.min().y),
            f64::min(box0.min().z, box1.min().z),
        );
        let big = Point3::new(
            f64::max(box0.max().x, box1.max().x),
            f64::max(box0.max().y, box1.max().y),
            f64::max(box0.max().z, box1.max().z),
        );
        Aabb::new(small, big)
    }
}
