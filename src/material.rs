use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec3::{Color, Vec3},
};
use derive_more::Constructor;

pub(crate) trait Material: std::fmt::Debug {
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

/// Reflective metal
#[derive(Debug, Clone, Copy)]
pub(crate) struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub(crate) fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = Vec3::reflect(_r_in.direction().to_unit(), rec.normal);
        let scattered = Ray::new(rec.p, reflected + self.fuzz * Vec3::random_in_unit_sphere());
        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

/// Dielectric metals (glass, water, etc)
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dielectric {
    /// Index of refraction
    ir: f64,
}

impl Dielectric {
    pub(crate) fn new(ir: f64) -> Self {
        Self { ir }
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().to_unit();
        let refracted = Vec3::refract(unit_direction, rec.normal, refraction_ratio);

        let scattered = Ray::new(rec.p, refracted);
        Some((attenuation, scattered))
    }
}
