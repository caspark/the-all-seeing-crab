use std::rc::Rc;

use crate::{
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HitRecord {
    pub t: f64,
    pub p: Point3,
    pub normal: Vec3,
    pub front_face: bool,
}

impl HitRecord {
    pub(crate) fn new(t: f64, r: &Ray, outward_normal: Vec3) -> HitRecord {
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
        }
    }
}

pub(crate) trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Clone, Default)]
pub(crate) struct HittableList {
    pub objects: Vec<Rc<Box<dyn Hittable>>>,
}

impl HittableList {
    pub(crate) fn clear(&mut self) {
        self.objects.clear();
    }

    pub(crate) fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(Rc::new(object))
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut best_hit: Option<HitRecord> = None;

        for object in self.objects.iter() {
            let new_t_max = best_hit.map_or(t_max, |h| h.t);
            if let Some(new_hit) = object.hit(r, t_min, new_t_max) {
                best_hit = Some(new_hit);
            }
        }

        best_hit
    }
}
