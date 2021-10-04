mod camera;
mod color;
mod hittable;
mod material;
mod ray;
mod sphere;
mod util;
mod vec3;

use material::{Dielectric, Material, Metal};
use rayon::prelude::*;
use std::{env, f64::INFINITY, rc::Rc};
use util::random_double;

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

fn create_fixed_scene() -> HittableList {
    let mut world = HittableList::default();

    let material_ground = Rc::new(DiffuseLambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(DiffuseLambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        -0.45,
        material_left,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

    world
}

fn create_random_scene() -> HittableList {
    let mut world = HittableList::default();

    let material_ground = Rc::new(DiffuseLambertian::new(Color::new(0.8, 0.8, 0.0)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_ground,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let a = a as f64;
            let b = b as f64;

            let choose_mat = random_double(0.0, 1.0);
            let center = Point3::new(
                a + 0.9 * random_double(0.0, 1.0),
                0.2,
                b + 0.9 * random_double(0.0, 1.0),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let material: Rc<dyn Material> = if choose_mat < 0.8 {
                    let albedo = Color::random(0.0, 1.0) * Color::random(0.0, 1.0);
                    Rc::new(DiffuseLambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.0);
                    let fuzz = random_double(0.0, 0.5);
                    Rc::new(Metal::new(albedo, fuzz))
                } else {
                    Rc::new(Dielectric::new(1.5)) // 1.5 is glass
                };
                world.add(Box::new(Sphere::new(center, 0.2, material)));
            }
        }
    }

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        0.5,
        Rc::new(Dielectric::new(1.5)),
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        0.5,
        Rc::new(DiffuseLambertian::new(Color::new(0.1, 0.2, 0.5))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        0.5,
        Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0)),
    )));

    world
}

fn run(image_filename: &str) {
    // scene
    let generate_random_scene = true;

    // image & rendering
    let aspect_ratio: f64 = if generate_random_scene {
        3.0 / 2.0
    } else {
        16.0 / 9.0
    };
    let image_width: i32 = 400;
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
    let incremental_progress_display = true;
    println!(
        "Image is {width}x{height} (total {count} pixels), with {samples} samples per pixel & max depth of {depth}",
        width = image_width,
        height = image_height,
        count = image_pixel_count,
        samples = samples_per_pixel,
        depth = max_depth,
    );

    // camera
    let cam = {
        let (look_from, look_at) = if generate_random_scene {
            (Point3::new(13.0, 2.0, 3.0), Point3::new(0.0, 0.0, 0.0))
        } else {
            (Point3::new(3.0, 3.0, 2.0), Point3::new(0.0, 0.0, -1.0))
        };
        let vup = Vec3::new(0.0, 1.0, 0.0);
        let vfov = 20.0;
        let (focus_dist, aperture) = if generate_random_scene {
            (10.0, 0.1)
        } else {
            ((look_from - look_at).length(), 1.0)
        };
        Camera::new(
            look_from,
            look_at,
            vup,
            vfov,
            aspect_ratio,
            aperture,
            focus_dist,
        )
    };

    type RenderLine = (i32, Vec<RGB8>);
    let (tx, rx) = flume::unbounded::<RenderLine>();

    let render_image_fn = || {
        println!("Rendering w/ {} threads...", render_threads);

        (0..image_height)
            .rev()
            .collect::<Vec<_>>()
            .into_par_iter()
            .for_each_init(
                || {
                    if generate_random_scene {
                        create_random_scene()
                    } else {
                        create_fixed_scene()
                    }
                },
                |world, j| {
                    let mut line_pixels = Vec::with_capacity(image_width as usize);
                    for i in 0..image_width {
                        let mut pixel_color: Color = Color::zero();
                        for _ in 0..samples_per_pixel {
                            let u = (i as f64 + util::random_double_unit())
                                / (image_width as f64 - 1.0);
                            let v = (j as f64 + util::random_double_unit())
                                / (image_height as f64 - 1.0);
                            let r = cam.get_ray(u, v);
                            pixel_color += ray_color(r, world, render_mode);
                        }

                        let rgb8 = color_as_rgb8(pixel_color, samples_per_pixel);
                        line_pixels.push(rgb8);
                    }

                    tx.send((j, line_pixels)).unwrap();
                },
            );
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
                if incremental_progress_display && line_num % height_incr == 0 {
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
