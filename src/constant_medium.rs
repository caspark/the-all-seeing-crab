use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    texture::Texture,
    util::random_double,
    vec3::{Color, Vec3},
};

#[derive(Debug)]
pub(crate) struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    phase_function: Box<dyn Material>,
    neg_inv_density: f64,
}

impl ConstantMedium {
    pub(crate) fn new(
        boundary: Box<dyn Hittable>,
        texture: Box<dyn Texture>,
        density: f64,
    ) -> Self {
        Self {
            boundary,
            phase_function: Box::new(Isotropic::new(texture)),
            neg_inv_density: -1.0 / density,
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut rec1 = self.boundary.hit(r, -std::f64::MAX, std::f64::MAX)?;
        let mut rec2 = self.boundary.hit(r, rec1.t + 0.0001, std::f64::MAX)?;

        if rec1.t < t_min {
            rec1.t = t_min;
        }
        if rec2.t > t_max {
            rec2.t = t_max;
        }
        if rec1.t >= rec2.t {
            return None;
        }
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * random_double(0.0, 1.0).ln();

        if hit_distance > distance_inside_boundary {
            return None;
        }

        Some(HitRecord::new(
            rec1.t + hit_distance / ray_length,
            (rec1.u, rec1.v),
            r,
            Vec3::new(1.0, 0.0, 0.0), // arbitrarily chosen normal
            self.phase_function.as_ref(),
        ))
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<crate::aabb::Aabb> {
        self.boundary.bounding_box(time0, time1)
    }
}

#[derive(Debug)]
struct Isotropic {
    albedo: Box<dyn Texture>,
}

impl Isotropic {
    fn new(albedo: Box<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: Ray, hit: &HitRecord) -> Option<(Color, Ray)> {
        Some((
            self.albedo.value(hit.u, hit.v, hit.p),
            Ray::new(hit.p, Vec3::random_in_unit_sphere(), Some(r_in.time())),
        ))
    }
}
