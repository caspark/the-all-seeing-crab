use crate::{
    ray::Ray,
    util::{degrees_to_radians, random_double},
    vec3::{Point3, Vec3},
};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct CameraSettings {
    pub look_from: Point3,
    pub look_at: Point3,
    pub vup: Vec3,
    pub vfov: f64,
    pub focus_dist: f64,
    pub aperture: f64,
    pub time0: f64,
    pub time1: f64,
}

#[allow(dead_code)]
impl CameraSettings {
    pub(crate) fn look_from(self, look_from: Point3) -> Self {
        Self { look_from, ..self }
    }

    pub(crate) fn look_at(self, look_at: Point3) -> Self {
        Self { look_at, ..self }
    }

    pub(crate) fn vup(self, vup: Vec3) -> Self {
        Self { vup, ..self }
    }

    pub(crate) fn vfov(self, vfov: f64) -> Self {
        Self { vfov, ..self }
    }

    pub(crate) fn focusing_on(self, focus_dist: f64, aperture: f64) -> Self {
        Self {
            focus_dist,
            aperture,
            ..self
        }
    }

    pub(crate) fn time_range(self, time0: f64, time1: f64) -> Self {
        Self {
            time0,
            time1,
            ..self
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            look_from: Point3::new(13.0, 2.0, 3.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov: 20.0,
            focus_dist: 10.0,
            aperture: 0.0,
            time0: 0.0,
            time1: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub lens_radius: f64,
    /// Shutter open time
    pub time0: f64,
    /// Shutter close time
    pub time1: f64,
}

impl Camera {
    pub fn new(settings: CameraSettings, aspect_ratio: f64) -> Camera {
        println!(
            "Looking from {from} to {at}, with up = {vup}",
            from = settings.look_from,
            at = settings.look_at,
            vup = settings.vup
        );

        let theta = degrees_to_radians(settings.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height: f64 = 2.0 * h;
        let viewport_width: f64 = aspect_ratio * viewport_height;

        let w = (settings.look_from - settings.look_at).to_unit();
        let u = settings.vup.cross(w).to_unit();
        let v = w.cross(u);
        println!(
            "Camera viewport is {height}x{width} with FOV of {fov}",
            height = viewport_height,
            width = viewport_width,
            fov = settings.vfov,
        );
        println!(
            "FOV is {fov}, aperture is {aperture}, focus distance is {focus_dist}",
            fov = settings.vfov,
            aperture = settings.aperture,
            focus_dist = settings.focus_dist,
        );

        let origin: Point3 = settings.look_from;
        let horizontal = settings.focus_dist * viewport_width * u;
        let vertical = settings.focus_dist * viewport_height * v;
        let lower_left_corner =
            origin - horizontal / 2.0 - vertical / 2.0 - settings.focus_dist * w;
        let lens_radius = settings.aperture / 2.0;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius,
            time0: settings.time0,
            time1: settings.time1,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
            Some(random_double(self.time0, self.time1)),
        )
    }
}
