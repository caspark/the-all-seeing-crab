use crate::{
    hittable::HitRecord,
    ray::Ray,
    util::random_double,
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
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            self.albedo,
            Ray::new(rec.p, scatter_direction, Some(r_in.time())),
        ))
    }
}

/// True lambertian reflection
#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct DiffuseLambertian {
    albedo: Color,
}

impl Material for DiffuseLambertian {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            self.albedo,
            Ray::new(rec.p, scatter_direction, Some(r_in.time())),
        ))
    }
}

/// True lambertian reflection with arbitrary textures
#[derive(Debug, Constructor)]
pub(crate) struct DiffuseLambertianTexture {
    albedo: Box<dyn crate::texture::Texture>,
}

impl Material for DiffuseLambertianTexture {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            self.albedo.value(rec.u, rec.v, rec.p),
            Ray::new(rec.p, scatter_direction, Some(r_in.time())),
        ))
    }
}

/// Hemispherical scattering
#[derive(Debug, Clone, Copy, Constructor)]
pub(crate) struct DiffuseHemispherical {
    albedo: Color,
}

impl Material for DiffuseHemispherical {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = Vec3::random_in_hemisphere(rec.normal);

        // avoid degenerate scatter direction (avoid infinities and NaNs)
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((
            self.albedo,
            Ray::new(rec.p, scatter_direction, Some(r_in.time())),
        ))
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
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = Vec3::reflect(r_in.direction().to_unit(), rec.normal);
        let scattered = Ray::new(
            rec.p,
            reflected + self.fuzz * Vec3::random_in_unit_sphere(),
            Some(r_in.time()),
        );
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

impl Dielectric {
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // use schlick's approximation for reflectance
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
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
        let cos_theta = f64::min((-unit_direction).dot(rec.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract
            || Dielectric::reflectance(cos_theta, refraction_ratio) > random_double(0.0, 1.0)
        {
            Vec3::reflect(unit_direction, rec.normal)
        } else {
            Vec3::refract(unit_direction, rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction, Some(r_in.time()));
        Some((attenuation, scattered))
    }
}
