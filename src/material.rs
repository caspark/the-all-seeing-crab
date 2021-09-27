use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec3::{Color, Vec3},
};
use derive_more::Constructor;

pub(crate) trait Material: std::fmt::Debug {
    /// Returns the scattered ray
    fn scatter(&self, r_in: Ray, rec: HitRecord) -> Option<(Color, Ray)>;
}

#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct Lambertian {
    albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: Ray, rec: HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}
