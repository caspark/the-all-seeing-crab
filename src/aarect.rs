use derive_more::Constructor;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Debug, Constructor)]
pub(crate) struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for XyRect {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.origin().z) / r.direction().z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.origin().x + t * r.direction().x;
        let y = r.origin().y + t * r.direction().y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);

        Some(HitRecord::new(
            t,
            (u, v),
            r,
            Vec3::new(0.0, 0.0, 1.0),
            &*self.material,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<crate::aabb::Aabb> {
        Some(Aabb::new(
            // The bounding box must have non-zero width in each dimension, so pad the Z
            // dimension a small amount.
            Vec3::new(self.x0, self.y0, self.k - 0.0001),
            Vec3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}
