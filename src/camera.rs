use crate::{
    ray::Ray,
    util::degrees_to_radians,
    vec3::{Point3, Vec3},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
    ) -> Camera {
        println!(
            "Looking from {from} to {at}, with up = {vup}",
            from = lookfrom,
            at = lookat,
            vup = vup
        );

        let theta = degrees_to_radians(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height: f64 = 2.0 * h;
        let viewport_width: f64 = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).to_unit();
        let u = vup.cross(w).to_unit();
        let v = w.cross(u);
        println!(
            "Camera viewport is {height}x{width} with FOV of {fov}",
            height = viewport_height,
            width = viewport_width,
            fov = vfov,
        );

        let origin: Point3 = lookfrom;
        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
