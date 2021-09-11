mod ray;
mod vec3;

use std::env;

use ray::Ray;
use rgb::RGB8;
use vec3::{lerp, Color, Point3, Vec3};

fn hit_sphere(center: Point3, radius: f64, r: Ray) -> bool {
    let oc = r.origin() - center;
    let a = r.direction().dot(r.direction());
    let b = 2.0 * oc.dot(r.direction());
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant > 0.0
}

fn ray_color(r: Ray) -> Color {
    if hit_sphere(Point3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
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
        [ref exe] => print_usage_then_die(&exe, "output file expected as first argument"),
        [_, ref image_filename] => run(image_filename),
        [ref exe, _, ..] => print_usage_then_die(&exe, "Max one argument expected"),
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
    eprintln!(
        "Image is {width}x{height} (total {count} pixels)",
        width = image_width,
        height = image_height,
        count = image_pixel_count
    );

    // camera
    let viewport_height: f64 = 2.0;
    let viewport_width: f64 = aspect_ratio * viewport_height;
    let focal_length: f64 = 1.0;
    eprintln!(
        "Viewport is {height}x{width} w/ focal length {focal}",
        height = viewport_height,
        width = viewport_width,
        focal = focal_length
    );

    let origin: Point3 = Point3::default();
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    let mut image_buffer = Vec::<RGB8>::with_capacity(image_pixel_count);
    for j in (0..image_height).rev() {
        if j % 100 == 0 {
            eprintln!("Scanlines remaining: {j}", j = j);
        }
        for i in 0..image_width {
            let u = i as f64 / (image_width as f64 - 1.0);
            let v = j as f64 / (image_height as f64 - 1.0);
            let r = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );

            let pixel: Color = ray_color(r);
            image_buffer.push(RGB8 {
                r: (pixel.x * 255.9999) as u8,
                g: (pixel.y * 255.9999) as u8,
                b: (pixel.z * 255.9999) as u8,
            });
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
