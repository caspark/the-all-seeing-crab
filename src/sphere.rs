use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use derive_more::Constructor;

#[derive(Debug, Clone, Constructor)]
pub(crate) struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Material,
}

impl Hittable for Sphere {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = oc.dot(r.direction());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);
        let outward_normal: Vec3 = (p - self.center) / self.radius;
        Some(HitRecord::new(t, r, outward_normal, &self.material))
    }
}
