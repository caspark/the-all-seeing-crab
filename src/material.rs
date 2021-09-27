use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec3::{Color, Vec3},
};
use derive_more::Constructor;

pub(crate) trait Material: std::fmt::Debug + Sync + Send {
    /// Returns the scattered ray
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

/// Bias of having light bounce towards the normal
#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct DiffuseHack {
    albedo: Color,
}

impl Material for DiffuseHack {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

/// True lambertian reflection
#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct DiffuseLambertian {
    albedo: Color,
}

impl Material for DiffuseLambertian {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

/// Hemispherical scattering
#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct DiffuseHemispherical {
    albedo: Color,
}

impl Material for DiffuseHemispherical {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = Vec3::random_in_hemisphere(rec.normal);

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}
