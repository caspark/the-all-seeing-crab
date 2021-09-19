mod camera;
mod color;
mod hittable;
mod ray;
mod sphere;
mod vec3;

use std::{env, f64::INFINITY};

use hittable::{Hittable, HittableList};
use ray::Ray;
use rgb::RGB8;
use vec3::{lerp, Color, Point3};

use crate::{camera::Camera, color::color_as_rgb8, sphere::Sphere};

fn random_double(min: f64, max: f64) -> f64 {
    min + (max - min) * rand::random::<f64>()
}

fn ray_color(r: &Ray, world: &HittableList) -> Color {
    if let Some(rec) = world.hit(r, 0.0, INFINITY) {
        return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
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
    eprintln!(
        "Image is {width}x{height} (total {count} pixels), with {samples} samples per pixel",
        width = image_width,
        height = image_height,
        count = image_pixel_count,
        samples = samples_per_pixel,
    );

    // world
    let mut world = HittableList::default();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // camera
    let cam = Camera::new(aspect_ratio);

    let mut image_buffer = Vec::<RGB8>::with_capacity(image_pixel_count);
    for j in (0..image_height).rev() {
        if j % 100 == 0 {
            eprintln!("Scanlines remaining: {j}", j = j);
        }
        for i in 0..image_width {
            let mut pixel_color: Color = Color::zero();
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + random_double(0.0, 1.0)) / (image_width as f64 - 1.0);
                let v = (j as f64 + random_double(0.0, 1.0)) / (image_height as f64 - 1.0);
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world);
            }

            image_buffer.push(color_as_rgb8(pixel_color, samples_per_pixel));
        }
    }
    debug_assert_eq!(image_buffer.len(), image_pixel_count);

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
