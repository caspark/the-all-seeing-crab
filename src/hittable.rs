use crate::{
    aabb::Aabb,
    material::Material,
    ray::Ray,
    util::degrees_to_radians,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone)]
pub(crate) struct HitRecord<'m> {
    /// How far along the ray the hit happened
    pub t: f64,
    /// The u texture coordinate of the hit
    pub u: f64,
    /// The v texture coordinate of the hit
    pub v: f64,
    /// Point (location) that the hit occurred at
    pub p: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub mat_ptr: &'m dyn Material,
}

impl HitRecord<'_> {
    pub(crate) fn new(
        t: f64,
        (u, v): (f64, f64),
        r: Ray,
        outward_normal: Vec3,
        material: &dyn Material,
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
            u,
            v,
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

#[derive(Debug, Default)]
pub(crate) struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object)
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut best_hit: Option<HitRecord> = None;

        for object in self.objects.iter() {
            let new_t_max = best_hit.as_ref().map_or(t_max, |h| h.t);
            if let Some(new_hit) = object.hit(r, t_min, new_t_max) {
                best_hit = Some(new_hit);
            }
        }

        best_hit
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let mut result = None;
        for object in self.objects.iter() {
            if let Some(new_box) = object.bounding_box(time0, time1) {
                result = Some(if let Some(existing) = result {
                    Aabb::surrounding_box(existing, new_box)
                } else {
                    new_box
                })
            } else {
                return None;
            }
        }
        result
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Translate<H: Hittable> {
    offset: Vec3,
    obj: H,
}

impl<H: Hittable> Translate<H> {
    pub(crate) fn new(offset: Vec3, obj: H) -> Self {
        Self { offset, obj }
    }
}

impl<H: Hittable> Hittable for Translate<H> {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let moved_r = Ray::new(r.origin() - self.offset, r.direction(), Some(r.time()));
        self.obj.hit(moved_r, t_min, t_max).map(|h| HitRecord {
            p: h.p + self.offset,
            ..h
        })
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.obj
            .bounding_box(time0, time1)
            .map(|b| Aabb::new(b.min() + self.offset, b.max() + self.offset))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RotateY<H: Hittable> {
    obj: H,
    sin_theta: f64,
    cos_theta: f64,
    bounding_box: Option<Aabb>,
}

impl<H: Hittable> RotateY<H> {
    pub(crate) fn new(angle: f64, obj: H) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let mut min = Vec3::new(std::f64::MAX, std::f64::MAX, std::f64::MAX);
        let mut max = Vec3::new(std::f64::MIN, std::f64::MIN, std::f64::MIN);

        Self {
            bounding_box: {
                obj.bounding_box(0.0, 1.0).map(|bbox| {
                    for i in 0..2 {
                        for j in 0..2 {
                            for k in 0..2 {
                                let x = i as f64 * bbox.max().x + (1 - i) as f64 * bbox.min().x;
                                let y = j as f64 * bbox.max().y + (1 - j) as f64 * bbox.min().y;
                                let z = k as f64 * bbox.max().z + (1 - k) as f64 * bbox.min().z;

                                let newx = cos_theta * x + sin_theta * z;
                                let newz = -sin_theta * x + cos_theta * z;

                                let t = Vec3::new(newx, y, newz);
                                for c in 0..3 {
                                    min[c] = min[c].min(t[c]);
                                    max[c] = max[c].max(t[c]);
                                }
                            }
                        }
                    }
                    Aabb::new(min, max)
                })
            },
            obj,
            sin_theta,
            cos_theta,
        }
    }
}

impl<H: Hittable> Hittable for RotateY<H> {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut origin = r.origin();
        let mut direction = r.direction();

        origin.x = self.cos_theta * r.origin().x - self.sin_theta * r.origin().z;
        origin.z = self.sin_theta * r.origin().x + self.cos_theta * r.origin().z;

        direction.x = self.cos_theta * r.direction().x - self.sin_theta * r.direction().z;
        direction.z = self.sin_theta * r.direction().x + self.cos_theta * r.direction().z;

        let rotated_r = Ray::new(origin, direction, Some(r.time()));

        self.obj.hit(rotated_r, t_min, t_max).map(|rec| HitRecord {
            p: Vec3::new(
                self.cos_theta * rec.p.x + self.sin_theta * rec.p.z,
                rec.p.y,
                -self.sin_theta * rec.p.x + self.cos_theta * rec.p.z,
            ),
            normal: Vec3::new(
                self.cos_theta * rec.normal.x + self.sin_theta * rec.normal.z,
                rec.normal.y,
                -self.sin_theta * rec.normal.x + self.cos_theta * rec.normal.z,
            ),
            ..rec
        })
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        self.bounding_box
    }
}
