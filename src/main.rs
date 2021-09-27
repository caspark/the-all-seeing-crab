mod camera;
mod color;
mod hittable;
mod ray;
mod sphere;
mod util;
mod vec3;

use std::{env, f64::INFINITY};

use hittable::{Hittable, HittableList};
use ray::Ray;
use rgb::RGB8;
use vec3::{lerp, Color, Point3};

use crate::{
    camera::Camera,
    color::{color_as_rgb8, rgb8_as_terminal_char},
    sphere::Sphere,
};

#[allow(dead_code)]
enum RayColorMode {
    /// shade as single purely matte color
    Solid { color: Color },
    /// shade by assuming the normal is the color
    ShadeNormal,
    /// shade based on distance from camera
    Depth { max_t: f64 },
    /// bias of having light bounce towards the normal
    DiffuseHack { depth: i32 },
    /// lambertian reflection
    DiffuseLambertian { depth: i32 },
    /// hemispherical scattering
    DiffuseAlternative { depth: i32 },
}

fn ray_color(r: Ray, world: &HittableList, mode: RayColorMode) -> Color {
    let curr_depth = {
        match mode {
            RayColorMode::DiffuseHack { depth } => Some(depth),
            RayColorMode::DiffuseLambertian { depth } => Some(depth),
            RayColorMode::DiffuseAlternative { depth } => Some(depth),
            _ => None,
        }
    };
    if curr_depth.map_or(false, |d| d <= 0) {
        return Color::zero();
    }

    if let Some(rec) = world.hit(&r, 0.001, INFINITY) {
        return match mode {
            RayColorMode::Solid { color } => color,
            RayColorMode::ShadeNormal => 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0)),
            RayColorMode::Depth { max_t } => Color::one() - rec.t / max_t * Color::one(),
            RayColorMode::DiffuseHack { depth } => {
                let target: Point3 = rec.p + rec.normal + Point3::random_in_unit_sphere();
                0.5 * ray_color(
                    Ray::new(rec.p, target - rec.p),
                    world,
                    RayColorMode::DiffuseHack { depth: depth - 1 },
                )
            }
            RayColorMode::DiffuseLambertian { depth } => {
                let target: Point3 = rec.p + rec.normal + Point3::random_unit_vector();
                0.5 * ray_color(
                    Ray::new(rec.p, target - rec.p),
                    world,
                    RayColorMode::DiffuseLambertian { depth: depth - 1 },
                )
            }
            RayColorMode::DiffuseAlternative { depth } => {
                let target: Point3 = rec.p + Point3::random_in_hemisphere(rec.normal);
                0.5 * ray_color(
                    Ray::new(rec.p, target - rec.p),
                    world,
                    RayColorMode::DiffuseLambertian { depth: depth - 1 },
                )
            }
        };
    }

    let unit_direction = r.direction().to_unit();
    let t = 0.5 * (unit_direction.y + 1.0);
    let ground: Color = Color::new(1.0, 1.0, 1.0);
    let sky: Color = Color::new(0.5, 0.7, 1.0);
    lerp(t, ground, sky)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[..] {
        [] => panic!("Could not extract executable name as first arg"),
        [ref exe] => print_usage_then_die(exe, "output file expected as first argument"),
        [_, ref image_filename] => run(image_filename),
        [ref exe, _, ..] => print_usage_then_die(exe, "Max one argument expected"),
    };
}

fn print_usage_then_die(exe: &str, error: &str) {
    eprintln!("Error: {}", error);
    eprintln!("Usage:");
    eprintln!("    {} OUTPUT_FILE", exe);

    std::process::exit(1);
}

fn run(image_filename: &str) {
    // image
    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: i32 = 400;
    let image_height: i32 = (image_width as f64 / aspect_ratio) as i32;
    let image_pixel_count = (image_width * image_height) as usize;
    let samples_per_pixel = 100;
    let max_depth = 50;
    eprintln!(
        "Image is {width}x{height} (total {count} pixels), with {samples} samples per pixel & max depth of {depth}",
        width = image_width,
        height = image_height,
        count = image_pixel_count,
        samples = samples_per_pixel,
        depth = max_depth,
    );

    // world
    let mut world = HittableList::default();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // camera
    let cam = Camera::new(aspect_ratio);

    let (width_incr, height_incr) = {
        let indicator_width = 100.0 as i32;
        let indicator_height = (indicator_width as f64 / aspect_ratio / 2.0) as i32;
        (
            (image_width / indicator_width),
            (image_height / indicator_height),
        )
    };

    eprintln!("Rendering:");
    let mut image_buffer = Vec::<RGB8>::with_capacity(image_pixel_count);
    for j in (0..image_height).rev() {
        let showing_progress_for_this_line = j % height_incr == 0;

        for i in 0..image_width {
            let mut pixel_color: Color = Color::zero();
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + util::random_double_unit()) / (image_width as f64 - 1.0);
                let v = (j as f64 + util::random_double_unit()) / (image_height as f64 - 1.0);
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(
                    r,
                    &world,
                    // RayColorMode::ShadeNormal,
                    // RayColorMode::Depth { max_t: 2.0 },
                    // RayColorMode::DiffuseHack { depth: max_depth },
                    // RayColorMode::DiffuseLambertian { depth: max_depth },
                    RayColorMode::DiffuseAlternative { depth: max_depth },
                );
            }

            let rgb8 = color_as_rgb8(pixel_color, samples_per_pixel);
            image_buffer.push(rgb8);

            if showing_progress_for_this_line && i % width_incr == 0 {
                eprint!("{}", rgb8_as_terminal_char(rgb8));
            }
        }

        if showing_progress_for_this_line {
            eprintln!("");
        }
    }
    debug_assert_eq!(image_buffer.len(), image_pixel_count);
    eprintln!();

    eprintln!("Saving result to disk at {} as png...", image_filename);
    lodepng::encode_file(
        image_filename,
        &image_buffer,
        image_width as usize,
        image_height as usize,
        lodepng::ColorType::RGB,
        8,
    )
    .expect("Encoding result and saving to disk failed");

    eprintln!("Done.");
}
