use crate::{
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug)]
pub(crate) struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
}

pub(crate) trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
