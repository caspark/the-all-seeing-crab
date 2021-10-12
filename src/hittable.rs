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
    pub(crate) fn new<'m>(
        t: f64,
        r: Ray,
        outward_normal: Vec3,
        material: &'m dyn Material,
    ) -> HitRecord<'m> {
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

pub(crate) trait Hittable: std::fmt::Debug + Sync {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

// #[derive(Default)]
// pub(crate) struct HittableList {
//     pub objects: Vec<Box<dyn Hittable>>,
// }

// impl HittableList {
//     pub(crate) fn add(&mut self, object: Box<dyn Hittable>) {
//         self.objects.push(object)
//     }
// }

// impl Hittable for HittableList {
//     fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
//         let mut best_hit: Option<HitRecord> = None;

//         for object in self.objects.iter() {
//             let new_t_max = best_hit.as_ref().map_or(t_max, |h| h.t);
//             if let Some(new_hit) = object.hit(r, t_min, new_t_max) {
//                 best_hit = Some(new_hit);
//             }
//         }

//         best_hit
//     }

//     fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
//         // let mut result = None;
//         // for object in self.objects.iter() {
//         //     if let Some(new_box) = object.bounding_box(time0, time1) {
//         //         result = Some(if let Some(existing) = result {
//         //             Aabb::surrounding_box(existing, new_box)
//         //         } else {
//         //             new_box
//         //         })
//         //     } else {
//         //         return None;
//         //     }
//         // }
//         // result

//         self.objects
//             .iter()
//             .map(|o| o.bounding_box(time0, time1))
//             .fold(None, |maybe_a, maybe_b| match (maybe_a, maybe_b) {
//                 (_, None) => None, // any missing aabb -> the whole list has no aabb
//                 (None, b) => b,
//                 (Some(a), Some(b)) => Some(Aabb::surrounding_box(a, b)),
//             })
//     }
// }
