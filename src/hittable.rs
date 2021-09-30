use std::rc::Rc;

use crate::{
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone)]
pub(crate) struct HitRecord {
    /// How far along the ray the hit happened
    pub t: f64,
    /// Point (location) that the hit occurred at
    pub p: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub mat_ptr: Rc<dyn Material>,
}

impl HitRecord {
    pub(crate) fn new(
        t: f64,
        r: &Ray,
        outward_normal: Vec3,
        material: Rc<dyn Material>,
    ) -> HitRecord {
        let p = r.at(t);
        let front_face = r.direction().dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        HitRecord {
            t,
            p,
            front_face,
            normal,
            mat_ptr: material,
        }
    }
}

pub(crate) trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Default)]
pub(crate) struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub(crate) fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object)
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut best_hit: Option<HitRecord> = None;

        for object in self.objects.iter() {
            let new_t_max = best_hit.as_ref().map_or(t_max, |h| h.t);
            if let Some(new_hit) = object.hit(r, t_min, new_t_max) {
                best_hit = Some(new_hit);
            }
        }

        best_hit
    }
}
