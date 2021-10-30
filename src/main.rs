#![deny(clippy::all)] // make all clippy warnings into errors

mod aabb;
mod bvh_node;
mod camera;
mod color;
mod hittable;
mod material;
mod perlin;
mod ray;
mod sphere;
mod texture;
mod ui;
mod util;
mod vec3;

use material::DiffuseLambertianTexture;
use perlin::Perlin;
use rgb::RGB8;
use std::{env, f64::INFINITY};
use texture::{CheckerTexture, ColorTexture, MarbleTexture, NoiseTexture, TurbulenceTexture};

use crate::{
    bvh_node::BvhNode,
    camera::Camera,
    color::color_as_rgb8,
    hittable::Hittable,
    material::{Dielectric, DiffuseLambertian, Material, Metal},
    ray::Ray,
    sphere::Sphere,
    util::random_double,
    vec3::{lerp, Color, Point3, Vec3},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
enum RenderScene {
    ThreeBody,
    ManyBalls,
    CheckersColliding,
    PerlinNoise,
}

impl RenderScene {
    fn default_camera_settings(&self) -> CameraSettings {
        match self {
            RenderScene::ThreeBody => {
                let look_from = Point3::new(3.0, 3.0, 2.0);
                let look_at = Point3::new(0.0, 0.0, -1.0);
                CameraSettings {
                    look_from,
                    look_at,
                    vup: Vec3::new(0.0, 1.0, 0.0),
                    vfov: 20.0,
                    focus_dist: (look_from - look_at).length(),
                    aperture: 0.25,
                    time0: 0.0,
                    time1: 0.0,
                }
            }
            RenderScene::ManyBalls => CameraSettings {
                look_from: Point3::new(13.0, 2.0, 3.0),
                look_at: Point3::new(0.0, 0.0, 0.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
                vfov: 20.0,
                focus_dist: 10.0,
                aperture: 0.1,
                time0: 0.0,
                time1: 1.0,
            },
            RenderScene::CheckersColliding => CameraSettings {
                look_from: Point3::new(13.0, 2.0, 3.0),
                look_at: Point3::new(0.0, 0.0, 0.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
                vfov: 20.0,
                focus_dist: 10.0,
                aperture: 0.1,
                time0: 0.0,
                time1: 0.0,
            },
            RenderScene::PerlinNoise => CameraSettings {
                look_from: Point3::new(13.0, 2.0, 3.0),
                look_at: Point3::new(0.0, 1.0, 0.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
                vfov: 20.0,
                focus_dist: 10.0,
                aperture: 0.0,
                time0: 0.0,
                time1: 0.0,
            },
        }
    }

    fn create_world(&self) -> BvhNode {
        match self {
            RenderScene::ThreeBody => create_fixed_scene(),
            RenderScene::ManyBalls => create_random_scene(),
            RenderScene::CheckersColliding => create_checkers_colliding_scene(),
            RenderScene::PerlinNoise => {
                let mut world = Vec::new();

                let noise = Perlin::new();

                let material_ground = Box::new(DiffuseLambertianTexture::new(Box::new(
                    NoiseTexture::new(noise.clone(), 4.0),
                )));
                let material_center = Box::new(DiffuseLambertianTexture::new(Box::new(
                    MarbleTexture::new(noise.clone(), 4.0, 5),
                )));
                let material_right = Box::new(DiffuseLambertianTexture::new(Box::new(
                    TurbulenceTexture::new(noise, 4.0, 5),
                )));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, -1000.0, 0.0),
                    1000.0,
                    material_ground,
                )) as Box<dyn Hittable>);
                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 2.0, 0.0),
                    2.0,
                    material_center,
                )) as Box<dyn Hittable>);
                world.push(Box::new(Sphere::stationary(
                    Point3::new(-1.0, 2.0, -2.0),
                    2.0,
                    material_right,
                )) as Box<dyn Hittable>);

                BvhNode::new(world, 0.0, 0.0)
            }
        }
    }
}

impl Default for RenderScene {
    fn default() -> Self {
        RenderScene::ThreeBody
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
struct CameraSettings {
    look_from: Point3,
    look_at: Point3,
    vup: Vec3,
    vfov: f64,
    focus_dist: f64,
    aperture: f64,
    time0: f64,
    time1: f64,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            look_from: Point3::new(13.0, 2.0, 3.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            vfov: 20.0,
            focus_dist: 10.0,
            aperture: 0.1,
            time0: 0.0,
            time1: 1.0,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct RenderConfig {
    image_width: usize,
    image_height: usize,
    samples_per_pixel: u32,
    render_mode: RayColorMode,
    scene: RenderScene,
    output_filename: String,
}

impl RenderConfig {
    pub(crate) fn image_pixel_count(&self) -> usize {
        self.image_width * self.image_height
    }

    pub(crate) fn aspect_ratio(&self) -> f64 {
        self.image_width as f64 / self.image_height as f64
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        let aspect_ratio = 16.0 / 9.0;
        let image_width = 400;
        Self {
            image_width,
            image_height: (image_width as f64 / aspect_ratio) as usize,
            samples_per_pixel: 100,
            render_mode: {
                // RayColorMode::BlockColor {
                //     color: Color::new(255.0, 0.0, 0.0),
                // }
                // RayColorMode::ShadeNormal
                // RayColorMode::Depth { max_t: 1.0 }
                RayColorMode::Material { depth: 50 }
            },
            scene: Default::default(),
            output_filename: "target/output.png".to_owned(),
        }
    }
}

enum RenderCommand {
    Render {
        config: RenderConfig,
        cam_settings: CameraSettings,
    },
}

enum RenderResult {
    Reset {
        image_width: usize,
        image_height: usize,
    },
    ImageLine {
        line_num: usize,
        line_pixels: Vec<RGB8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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

fn ray_color(r: Ray, world: &dyn Hittable, mode: RayColorMode) -> Color {
    if let RayColorMode::Material { depth } = mode {
        if depth <= 0 {
            return Color::zero();
        }
    }

    if let Some(rec) = world.hit(r, 0.001, INFINITY) {
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

fn run(image_filename: &str) {
    let (command_tx, command_rx) = flume::unbounded::<RenderCommand>();
    let (result_tx, result_rx) = flume::unbounded::<RenderResult>();

    // start a background thread to handle rendering, but drop its handle so we don't wait for it
    // to finish
    drop(std::thread::spawn(move || {
        rayon::ThreadPoolBuilder::new()
            .num_threads(16)
            .build()
            .expect("should be able to build threadpool")
            .install(|| {
                run_render_loop(command_rx, result_tx);
            });
    }));

    let app = ui::TemplateApp::new(image_filename, command_tx, result_rx);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

fn print_usage_then_die(exe: &str, error: &str) {
    eprintln!("Error: {}", error);
    eprintln!("Usage:");
    eprintln!("    {} OUTPUT_FILE", exe);

    std::process::exit(1);
}

fn create_fixed_scene() -> BvhNode {
    let mut world = Vec::new();

    let material_ground = Box::new(DiffuseLambertianTexture::new(Box::new(
        CheckerTexture::from_colors(10.0, Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9)),
    )));
    let material_center = Box::new(DiffuseLambertianTexture::new(Box::new(
        ColorTexture::from_rgb(0.1, 0.2, 0.5),
    )));
    let material_left = Box::new(Dielectric::new(1.5));
    let material_right = Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )) as Box<dyn Hittable>);
    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    )) as Box<dyn Hittable>);
    world.push(Box::new(Sphere::stationary(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left.clone(),
    )) as Box<dyn Hittable>);
    world.push(Box::new(Sphere::stationary(
        Point3::new(-1.0, 0.0, -1.0),
        -0.45,
        material_left,
    )) as Box<dyn Hittable>);
    world.push(Box::new(Sphere::stationary(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )) as Box<dyn Hittable>);

    BvhNode::new(world, 0.0, 0.0)
}

fn create_random_scene() -> BvhNode {
    let mut world = Vec::new();

    let material_ground = Box::new(DiffuseLambertian::new(Color::new(0.8, 0.8, 0.0)));
    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_ground,
    )) as Box<dyn Hittable>);

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
                let material: Box<dyn Material + Send + Sync> = if choose_mat < 0.8 {
                    let albedo = Color::random(0.0, 1.0) * Color::random(0.0, 1.0);
                    Box::new(DiffuseLambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.0);
                    let fuzz = random_double(0.0, 0.5);
                    Box::new(Metal::new(albedo, fuzz))
                } else {
                    Box::new(Dielectric::new(1.5)) // 1.5 is glass
                };
                world.push(if choose_mat < 0.4 {
                    Box::new(Sphere::stationary(center, 0.2, material)) as Box<dyn Hittable>
                } else {
                    let center2 = center + Vec3::new(0.0, random_double(0.0, 0.5), 0.0);
                    Box::new(Sphere::moving(center, center2, 0.0, 1.0, 0.2, material))
                        as Box<dyn Hittable>
                });
            }
        }
    }

    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Box::new(Dielectric::new(1.5)),
    )));

    world.push(Box::new(Sphere::stationary(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Box::new(DiffuseLambertian::new(Color::new(0.1, 0.2, 0.5))),
    )));
    world.push(Box::new(Sphere::stationary(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0)),
    )));

    BvhNode::new(world, 0.0, 1.0)
}

fn create_checkers_colliding_scene() -> BvhNode {
    let mut world = Vec::new();

    let light = Color::new(0.2, 0.3, 0.1);
    let dark = Color::new(0.9, 0.9, 0.9);

    let material_top = Box::new(DiffuseLambertianTexture::new(Box::new(
        CheckerTexture::from_colors(10.0, light, dark),
    )));
    let material_bottom = Box::new(DiffuseLambertianTexture::new(Box::new(
        CheckerTexture::from_colors(10.0, dark, light),
    )));

    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        material_top,
    )) as Box<dyn Hittable>);
    world.push(Box::new(Sphere::stationary(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        material_bottom,
    )) as Box<dyn Hittable>);

    BvhNode::new(world, 0.0, 0.0)
}

fn run_render_loop(
    render_command_rx: flume::Receiver<RenderCommand>,
    render_result_tx: flume::Sender<RenderResult>,
) {
    let mut abort_switch: Option<std::sync::Arc<std::sync::atomic::AtomicBool>> = Some(
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    );
    loop {
        match render_command_rx.recv() {
            Err(flume::RecvError::Disconnected) => break, // nothing to do, just quit quietly

            Ok(RenderCommand::Render {
                config,
                cam_settings,
            }) => {
                render_result_tx
                    .send(RenderResult::Reset {
                        image_height: config.image_height,
                        image_width: config.image_width,
                    })
                    .ok()
                    .expect("sending Reset should succeed");

                // abort any in progress render
                if let Some(ref mut should_abort) = abort_switch {
                    // cause a possible past render thread which is watching this flag to stop rendering
                    should_abort.store(true, std::sync::atomic::Ordering::SeqCst);
                    // set up the flag for the render thread we're about to kick off
                    abort_switch = Some(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
                        false,
                    )));
                }

                let world = config.scene.create_world();

                let cam = Camera::new(cam_settings, config.aspect_ratio());

                let render_result_tx = render_result_tx.clone();
                let abort_checker = abort_switch.as_ref().unwrap().clone();
                // drop the thread's join handle so that it runs in the background until rendering is done
                std::mem::drop(std::thread::spawn(move || {
                    use rayon::prelude::*;
                    (0..config.image_height)
                        .rev()
                        .collect::<Vec<_>>()
                        .into_par_iter()
                        .for_each(|j| {
                            if abort_checker.load(std::sync::atomic::Ordering::SeqCst) {
                                // don't do the work of rendering if it's not useful
                                return;
                            }

                            let mut line_pixels = Vec::with_capacity(config.image_width as usize);
                            for i in 0..config.image_width {
                                let mut pixel_color: Color = Color::zero();
                                for _ in 0..config.samples_per_pixel {
                                    let u = (i as f64 + util::random_double_unit())
                                        / (config.image_width as f64 - 1.0);
                                    let v = (j as f64 + util::random_double_unit())
                                        / (config.image_height as f64 - 1.0);
                                    let r = cam.get_ray(u, v);
                                    pixel_color += ray_color(r, &world, config.render_mode);
                                }

                                let rgb8 = color_as_rgb8(pixel_color, config.samples_per_pixel);
                                line_pixels.push(rgb8);
                            }

                            if abort_checker.load(std::sync::atomic::Ordering::SeqCst) {
                                // don't send calculated image data if we should have already aborted
                                return;
                            }
                            render_result_tx
                                .send(RenderResult::ImageLine {
                                    line_num: j,
                                    line_pixels,
                                })
                                .ok()
                                .unwrap();
                        });
                }));
            }
        }
    }
}
