use derive_more::Constructor;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug)]
pub(crate) struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl XyRect {
    pub(crate) fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        material: Box<dyn Material>,
    ) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            material,
        }
    }
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
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}

#[derive(Debug, Constructor)]
pub(crate) struct XzRect {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for XzRect {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.origin().y) / r.direction().y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.origin().x + t * r.direction().x;
        let z = r.origin().z + t * r.direction().z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (z - self.z0) / (self.z1 - self.z0);

        Some(HitRecord::new(
            t,
            (u, v),
            r,
            Vec3::new(0.0, 1.0, 0.0),
            &*self.material,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<crate::aabb::Aabb> {
        Some(Aabb::new(
            // The bounding box must have non-zero width in each dimension, so pad the Y
            // dimension a small amount.
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1),
        ))
    }
}

#[derive(Debug, Constructor)]
pub(crate) struct YzRect {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    material: Box<dyn Material>,
}

impl Hittable for YzRect {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.origin().x) / r.direction().x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = r.origin().y + t * r.direction().y;
        let z = r.origin().z + t * r.direction().z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let u = (y - self.y0) / (self.y1 - self.y0);
        let v = (z - self.z0) / (self.z1 - self.z0);

        Some(HitRecord::new(
            t,
            (u, v),
            r,
            Vec3::new(1.0, 0.0, 0.0),
            &*self.material,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<crate::aabb::Aabb> {
        Some(Aabb::new(
            // The bounding box must have non-zero width in each dimension, so pad the X
            // dimension a small amount.
            Point3::new(self.k - 0.0001, self.y0, self.z0),
            Point3::new(self.k + 0.0001, self.y1, self.z1),
        ))
    }
}
