mod camera;
mod color;
mod hittable;
mod material;
mod ray;
mod sphere;
mod util;
mod vec3;

use material::{Dielectric, Metal};
use rayon::prelude::*;
use std::{env, f64::INFINITY, rc::Rc};

use hittable::{Hittable, HittableList};
use ray::Ray;
use rgb::RGB8;
use vec3::{lerp, Color, Point3};

use crate::{
    camera::Camera,
    color::{color_as_rgb8, rgb8_as_terminal_char},
    material::DiffuseLambertian,
    sphere::Sphere,
    vec3::Vec3,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum RayColorMode {
    /// shade as single purely matte color
    BlockColor { color: Color },
    /// shade by assuming the normal is the color
    ShadeNormal,
    /// shade based on distance from camera
    Depth { max_t: f64 },
    /// use the assigned materials of each hittable object
    Material { depth: i32 },
}

fn ray_color(r: Ray, world: &HittableList, mode: RayColorMode) -> Color {
    if let RayColorMode::Material { depth } = mode {
        if depth <= 0 {
            return Color::zero();
        }
    }

    if let Some(rec) = world.hit(&r, 0.001, INFINITY) {
        return match mode {
            RayColorMode::BlockColor { color } => color,
            RayColorMode::ShadeNormal => 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0)),
            RayColorMode::Depth { max_t } => Color::one() - rec.t / max_t * Color::one(),
            RayColorMode::Material { depth } => {
                if let Some((attenuation, scattered)) = rec.mat_ptr.scatter(r, &rec) {
                    let new_depth = RayColorMode::Material { depth: depth - 1 };
                    return attenuation * ray_color(scattered, world, new_depth);
                } else {
                    return Color::zero();
                }
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

fn create_world() -> HittableList {
    let mut hittables = HittableList::default();

    let material_ground = Rc::new(DiffuseLambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(DiffuseLambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    hittables.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    hittables.add(Box::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    )));
    hittables.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )));
    hittables.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        -0.45,
        material_left,
    )));
    hittables.add(Box::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

    hittables
}

fn run(image_filename: &str) {
    // image & rendering
    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: i32 = 800;
    let image_height: i32 = (image_width as f64 / aspect_ratio) as i32;
    let image_pixel_count = (image_width * image_height) as usize;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let render_mode: RayColorMode = {
        // RayColorMode::BlockColor {
        //     color: Color::new(255.0, 0.0, 0.0),
        // }
        // RayColorMode::ShadeNormal
        // RayColorMode::Depth { max_t: 1.0 }
        RayColorMode::Material { depth: max_depth }
    };
    let render_threads = 16;
    let render_delay = std::time::Duration::from_millis(0);
    println!(
        "Image is {width}x{height} (total {count} pixels), with {samples} samples per pixel & max depth of {depth}",
        width = image_width,
        height = image_height,
        count = image_pixel_count,
        samples = samples_per_pixel,
        depth = max_depth,
    );

    // camera
    let cam = Camera::new(
        Point3::new(-2.0, 2.0, 1.0),
        Point3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        aspect_ratio,
    );

    type RenderLine = (i32, Vec<RGB8>);
    let (tx, rx) = flume::unbounded::<RenderLine>();

    let render_image_fn = || {
        println!("Rendering w/ {} threads...", render_threads);

        (0..image_height)
            .rev()
            .collect::<Vec<_>>()
            .into_par_iter()
            .for_each_init(create_world, |world, j| {
                let mut line_pixels = Vec::with_capacity(image_width as usize);
                for i in 0..image_width {
                    let mut pixel_color: Color = Color::zero();
                    for _ in 0..samples_per_pixel {
                        let u =
                            (i as f64 + util::random_double_unit()) / (image_width as f64 - 1.0);
                        let v =
                            (j as f64 + util::random_double_unit()) / (image_height as f64 - 1.0);
                        let r = cam.get_ray(u, v);
                        pixel_color += ray_color(r, world, render_mode);
                    }

                    let rgb8 = color_as_rgb8(pixel_color, samples_per_pixel);
                    line_pixels.push(rgb8);
                }

                tx.send((j, line_pixels)).unwrap();
            });
    };

    let output_image_fn = || {
        let mut image_buffer = vec![RGB8 { r: 0, g: 0, b: 0 }; image_pixel_count];
        {
            let progress_indicator_width = 100i32;
            let progress_indicator_height =
                (progress_indicator_width as f64 / aspect_ratio / 2.0) as i32;
            let width_incr = image_width / progress_indicator_width;
            let height_incr = image_height / progress_indicator_height;

            let mut progress_lines_written = 0;

            for _ in 0..image_height {
                let (line_num, pixels) = rx.recv().unwrap();
                let line_num = image_height - line_num - 1;
                assert_eq!(pixels.len(), image_width as usize);

                let offset_start = line_num as usize * image_width as usize;
                let offset_end = offset_start + image_width as usize;
                image_buffer[offset_start..offset_end].copy_from_slice(pixels.as_slice());

                // we only want to update the terminal render-in-progress display if the line we just
                // received is actually going to change the display
                if line_num % height_incr == 0 {
                    if progress_lines_written > 0 {
                        print!("{}", termion::cursor::Up(progress_lines_written));
                        progress_lines_written = 0;
                    }
                    for j in 0..(image_height as usize) {
                        let showing_progress_for_this_line = j % height_incr as usize == 0;
                        for i in 0..(image_width as usize) {
                            if showing_progress_for_this_line && i % width_incr as usize == 0 {
                                let c = rgb8_as_terminal_char(
                                    image_buffer[j * image_width as usize + i],
                                );
                                print!("{}", c);
                            }
                        }
                        if showing_progress_for_this_line {
                            println!();
                            progress_lines_written += 1;
                        }
                    }

                    if render_delay.as_millis() > 0 {
                        std::thread::sleep(render_delay);
                    }
                }
            }
            assert_eq!(image_buffer.len(), image_pixel_count);
        }

        println!(
            "Saving resulting image to disk at {} in PNG format...",
            image_filename
        );
        lodepng::encode_file(
            image_filename,
            &image_buffer,
            image_width as usize,
            image_height as usize,
            lodepng::ColorType::RGB,
            8,
        )
        .expect("Encoding result and saving to disk failed");

        println!("Done saving.");
    };

    let threadpool = rayon::ThreadPoolBuilder::new()
        // thread count has 1 added for the thread used to collect & display the results
        .num_threads(render_threads + 1)
        .build()
        .unwrap();
    threadpool.join(render_image_fn, output_image_fn);
}
