use crate::{
    hittable::HitRecord,
    ray::Ray,
    util::random_double,
    vec3::{Color, Vec3},
};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // some materials are obsoleted
pub(crate) enum Material {
    /// Bias of having light bounce towards the normal
    DiffuseHack { albedo: Color },

    /// True lambertian reflection
    DiffuseLambertian { albedo: Color },

    /// Hemispherical scattering
    DiffuseHemispherical { albedo: Color },

    /// Reflective metal
    Metal { albedo: Color, fuzz: f64 },

    /// Dielectric metals (glass, water, etc)
    Dielectric {
        /// Index of refraction
        ir: f64,
    },
}

impl Material {
    pub(crate) fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        match self {
            Material::DiffuseHack { albedo } => {
                let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere();

                // avoid degenerate scatter direction (avoid infinities and NaNs)
                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some((*albedo, Ray::new(rec.p, scatter_direction)))
            }
            Material::DiffuseLambertian { albedo } => {
                let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

                // avoid degenerate scatter direction (avoid infinities and NaNs)
                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some((*albedo, Ray::new(rec.p, scatter_direction)))
            }
            Material::DiffuseHemispherical { albedo } => {
                let mut scatter_direction = Vec3::random_in_hemisphere(rec.normal);

                // avoid degenerate scatter direction (avoid infinities and NaNs)
                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some((*albedo, Ray::new(rec.p, scatter_direction)))
            }
            Material::Metal { albedo, fuzz } => {
                let reflected = Vec3::reflect(r_in.direction().to_unit(), rec.normal);
                let scattered =
                    Ray::new(rec.p, reflected + (*fuzz) * Vec3::random_in_unit_sphere());
                if scattered.direction().dot(rec.normal) > 0.0 {
                    Some((*albedo, scattered))
                } else {
                    None
                }
            }
            Material::Dielectric { ir } => {
                let attenuation = Color::new(1.0, 1.0, 1.0);
                let refraction_ratio = if rec.front_face { 1.0 / ir } else { *ir };

                let unit_direction = r_in.direction().to_unit();
                let cos_theta = f64::min((-unit_direction).dot(rec.normal), 1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = refraction_ratio * sin_theta > 1.0;

                let direction = if cannot_refract
                    || reflectance(cos_theta, refraction_ratio) > random_double(0.0, 1.0)
                {
                    Vec3::reflect(unit_direction, rec.normal)
                } else {
                    Vec3::refract(unit_direction, rec.normal, refraction_ratio)
                };

                let scattered = Ray::new(rec.p, direction);
                Some((attenuation, scattered))
            }
        }
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    // use schlick's approximation for reflectance
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

// impl Metal {
//     pub(crate) fn new(albedo: Color, fuzz: f64) -> Self {
//         Self {
//             albedo,
//             fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
//         }
//     }
// }
