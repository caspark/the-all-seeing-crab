#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod aabb;
mod aarect;
mod box3d;
mod bvh_node;
mod camera;
mod color;
mod constant_medium;
mod hittable;
mod material;
mod perlin;
mod ray;
mod scenes;
mod sphere;
mod texture;
mod ui;
mod util;
mod vec3;

use camera::CameraSettings;
use rgb::RGB8;
use scenes::RenderScene;
use std::{env, f64::INFINITY};

use crate::{
    bvh_node::BvhNode,
    camera::Camera,
    color::color_as_rgb8,
    hittable::Hittable,
    ray::Ray,
    vec3::{lerp, Color},
};

#[derive(Debug)]
struct World {
    background: Option<Color>,
    node: BvhNode,
}

impl From<BvhNode> for World {
    fn from(node: BvhNode) -> Self {
        Self {
            node,
            background: Default::default(),
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
    display_actual_size: bool,
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
            render_mode: { RayColorMode::Material { depth: 50 } },
            scene: Default::default(),
            output_filename: "target/output.png".to_owned(),
            display_actual_size: true,
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

fn ray_color(r: Ray, background: Option<Color>, world: &dyn Hittable, mode: RayColorMode) -> Color {
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
                let emitted = rec.mat_ptr.emitted(rec.u, rec.v, rec.p);

                if let Some((attenuation, scattered)) = rec.mat_ptr.scatter(r, &rec) {
                    let new_depth = RayColorMode::Material { depth: depth - 1 };
                    return emitted
                        + attenuation * ray_color(scattered, background, world, new_depth);
                } else {
                    return emitted;
                }
            }
        };
    }

    background.unwrap_or_else(|| {
        let unit_direction = r.direction().to_unit();
        let t = 0.5 * (unit_direction.y + 1.0);
        let ground: Color = Color::new(1.0, 1.0, 1.0);
        let sky: Color = Color::new(0.5, 0.7, 1.0);
        lerp(t, ground, sky)
    })
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
                                    pixel_color += ray_color(
                                        r,
                                        world.background,
                                        &world.node,
                                        config.render_mode,
                                    );
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
