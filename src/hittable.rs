use crate::{
    aabb::Aabb,
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone)]
pub(crate) struct HitRecord<'m> {
    /// How far along the ray the hit happened
    pub t: f64,
    /// Point (location) that the hit occurred at
    pub p: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub mat_ptr: &'m dyn Material,
}

impl HitRecord<'_> {
    pub(crate) fn new(t: f64, r: Ray, outward_normal: Vec3, material: &dyn Material) -> HitRecord {
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

pub(crate) trait Hittable: std::fmt::Debug + Sync + Send {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}
