use std::{collections::HashMap, io::Write, ops::Rem};

use eframe::{
    egui::{self, TextureId},
    epi,
};
use rgb::RGB8;

use crate::{
    color::rgb8_as_terminal_char,
    vec3::{Color, Vec3},
    CameraSettings, RayColorMode, RenderCommand, RenderConfig, RenderResult, RenderScene,
};

#[derive(Debug, Default)]
struct UiData {
    last_render_width: usize,
    last_render_height: usize,
    last_render_lines_received: usize,
    last_render_pixels: Vec<RGB8>,
    last_render_tex: Option<TextureId>,

    terminal_initial_render_done: bool,
}

impl UiData {
    fn new(width: usize, height: usize) -> Self {
        Self {
            last_render_width: width,
            last_render_height: height,
            last_render_pixels: vec![RGB8 { r: 0, g: 0, b: 0 }; width * height],
            ..Default::default()
        }
    }

    fn rebuild_texture(&mut self, tex_allocator: &mut dyn eframe::epi::TextureAllocator) {
        if let Some(existing_tex) = self.last_render_tex {
            tex_allocator.free(existing_tex);
        }
        let tex_pixels = self
            .last_render_pixels
            .iter()
            .map(|rgb| egui::Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 255))
            .collect::<Vec<_>>();
        self.last_render_tex = Some(tex_allocator.alloc_srgba_premultiplied(
            (self.last_render_width, self.last_render_height),
            &tex_pixels,
        ));
    }

    fn clear_texture(&mut self, tex_allocator: &mut dyn eframe::epi::TextureAllocator) {
        if let Some(existing_tex) = self.last_render_tex {
            tex_allocator.free(existing_tex);
            self.last_render_tex = None;
        }
    }

    fn store_pixel_line(&mut self, line_num: usize, line_pixels: Vec<RGB8>) {
        assert_eq!(line_pixels.len(), self.last_render_width);
        assert!(self.last_render_lines_received < self.last_render_height);
        self.last_render_lines_received += 1;

        // update the image buffer
        let line_num = self.last_render_height - line_num - 1;
        let offset_start = line_num as usize * self.last_render_width;
        let offset_end = offset_start + self.last_render_width;
        self.last_render_pixels[offset_start..offset_end].copy_from_slice(line_pixels.as_slice());
    }

    fn render_terminal_progress_indicator(&mut self, settings: &TerminalSettings, line_num: usize) {
        use std::fmt::Write; // needed to use write! with strings

        let TerminalSettings {
            desired_width,
            desired_height,
        } = *settings;

        let height_ratio = self.last_render_height as f64 / desired_height as f64;
        let width_ratio = self.last_render_width as f64 / desired_width as f64;

        // Terminals are slow, so if we output every line to stdout then our app will end up blocking
        // writing on stdout, which will cause the UI thread to hang. Therefore we only output a line
        // if we know it will impact the resulting image.
        // Essentially this weird looking maths is attempting to do the inverse of
        // `(j as f64 * height_ratio) as usize;` - it is finding whether that will result in line_num
        // for any j from 0 to the desired terminal output height.
        // It was determined experimentally - if it breaks, it can be replaced with something like:
        // (0..settings.desired_height).map(|j| (j as f64 * height_ratio) as usize).find(line_num).is_some();
        let should_rerender =
            (height_ratio * 0.99999999999 + line_num as f64 + 1.0).rem(height_ratio) < 1.0;
        if should_rerender {
            // string sizing note: width + 1 char for newline on each line, plus an arbitrary 10 bytes
            // for the "move cursor up" terminal escape code we might have
            let mut output = String::with_capacity((desired_width + 1) * desired_height + 10);

            if self.terminal_initial_render_done {
                write!(output, "{}", termion::cursor::Up(desired_height as u16)).unwrap();
            }
            for j in 0..desired_height {
                let y = (j as f64 * height_ratio) as usize;
                for i in 0..desired_width {
                    let x = (i as f64 * width_ratio) as usize;
                    let pixel = self.last_render_pixels[y * self.last_render_width + x];
                    write!(output, "{}", rgb8_as_terminal_char(pixel)).unwrap();
                }
                writeln!(output).unwrap();
            }

            std::io::stdout()
                .lock()
                .write_all(output.as_bytes())
                .unwrap();

            self.terminal_initial_render_done = true;
        }
    }

    fn save_output_to_file(&self, output_filename: &str) {
        // make sure we got all the data we should have
        assert_eq!(
            self.last_render_pixels.len(),
            self.last_render_width * self.last_render_height
        );

        print!(
            "Saving completed image to disk at {} in PNG format...",
            output_filename
        );
        lodepng::encode_file(
            output_filename,
            &self.last_render_pixels,
            self.last_render_width,
            self.last_render_height,
            lodepng::ColorType::RGB,
            8,
        )
        .expect("Encoding result and saving to disk failed");

        println!(" done saving.");
    }

    fn complete(&self) -> bool {
        self.last_render_lines_received == self.last_render_height
    }

    fn percent_complete(&self) -> f32 {
        self.last_render_lines_received as f32 / self.last_render_height as f32
    }
}

#[derive(Clone, Copy, Debug)]
struct TerminalSettings {
    desired_width: usize,
    desired_height: usize,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            desired_width: 80,
            desired_height: 15,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DisplaySettings {
    display_actual_size: bool,
}

#[derive(Debug)]
pub struct TemplateApp {
    config: RenderConfig,
    data: Option<UiData>,
    display: DisplaySettings,

    scene_to_camera: HashMap<RenderScene, CameraSettings>,

    terminal_display: Option<TerminalSettings>,

    render_command_tx: flume::Sender<RenderCommand>,
    render_result_rx: flume::Receiver<RenderResult>,
}

impl TemplateApp {
    pub(crate) fn new(
        output_filename: &str,
        render_command_tx: flume::Sender<RenderCommand>,
        render_result_rx: flume::Receiver<RenderResult>,
    ) -> Self {
        let config = RenderConfig {
            output_filename: output_filename.to_owned(),
            ..Default::default()
        };
        TemplateApp {
            config,
            data: Default::default(),
            display: Default::default(),
            scene_to_camera: HashMap::new(),
            terminal_display: Some(TerminalSettings::default()),
            render_command_tx,
            render_result_rx,
        }
    }

    fn trigger_render(&self) {
        println!(
            "Triggering render of {width}x{height} image (total {count} pixels), with {samples} samples per pixel",
            width =self. config.image_width,
            height =self. config.image_height,
            count = self.config.image_pixel_count(),
            samples =self. config.samples_per_pixel,
        );

        self.render_command_tx
            .send(RenderCommand::Render {
                cam_settings: *self
                    .scene_to_camera
                    .get(&self.config.scene)
                    .to_owned()
                    .unwrap_or(&self.config.scene.default_camera_settings()),
                config: self.config.clone(),
            })
            .ok()
            .expect("render command send should succeed");
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "The All Seeing Crab"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        if let Some(storage) = _storage {
            self.config = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.trigger_render();
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.config);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        loop {
            match self.render_result_rx.try_recv() {
                Ok(RenderResult::Reset {
                    image_height,
                    image_width,
                }) => {
                    assert!(image_width > 0);
                    assert!(image_height > 0);

                    if let Some(ref mut d) = self.data {
                        d.clear_texture(frame.tex_allocator());
                    }
                    self.data = Some(UiData::new(image_width, image_height));
                }
                Ok(RenderResult::ImageLine {
                    line_num,
                    line_pixels,
                }) => {
                    let data = self
                        .data
                        .as_mut()
                        .expect("ui data must be present for storing pixels");

                    data.store_pixel_line(line_num, line_pixels);

                    if let Some(settings) = self.terminal_display {
                        data.render_terminal_progress_indicator(&settings, line_num);
                    }

                    if data.complete() {
                        data.save_output_to_file(self.config.output_filename.as_ref());
                    }
                    data.rebuild_texture(frame.tex_allocator());
                }
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    panic!("Rendering thread seems to have exited before UI!")
                }
            };
        }

        egui::SidePanel::left("config_panel")
            // .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 8.0);

                    ui.heading("Render Configuration");
                    egui::warn_if_debug_build(ui);
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label("Scene");
                        ui.vertical(|ui| {
                            ui.radio_value(
                                &mut self.config.scene,
                                RenderScene::ThreeBody,
                                "3 Body",
                            );
                            ui.radio_value(
                                &mut self.config.scene,
                                RenderScene::ManyBalls,
                                "Many Balls",
                            );
                            ui.radio_value(
                                &mut self.config.scene,
                                RenderScene::CheckersColliding,
                                "Checkers Colliding",
                            );
                            ui.radio_value(
                                &mut self.config.scene,
                                RenderScene::PerlinNoise,
                                "Perlin Noise",
                            );
                        });
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label("Save as");
                        ui.text_edit_singleline(&mut self.config.output_filename);
                    });
                    ui.end_row();

                    ui.vertical_centered_justified(|ui| {
                        let button = egui::widgets::Button::new("Render image!");
                        if ui.add(button).clicked() {
                            self.trigger_render();
                        }
                    });
                    ui.end_row();

                    ui.collapsing("Graphical display options", |ui| {
                        ui.checkbox(
                            &mut self.display.display_actual_size,
                            "Display render at actual 1:1 size",
                        );
                    });

                    ui.collapsing("Terminal display options", |ui| {
                        let mut terminal_progress = self.terminal_display.is_some();
                        ui.checkbox(&mut &mut terminal_progress, "Render progress in terminal");

                        if terminal_progress {
                            let settings = self.terminal_display.get_or_insert(Default::default());

                            ui.horizontal(|ui| {
                                ui.label("Render size of");
                                ui.add(
                                    egui::widgets::DragValue::new(&mut settings.desired_width)
                                        .suffix("w")
                                        .speed(1),
                                );
                                ui.label("by");
                                ui.add(
                                    egui::widgets::DragValue::new(&mut settings.desired_height)
                                        .suffix("h")
                                        .speed(1),
                                );
                                ui.label("chars");
                            });
                        } else {
                            self.terminal_display = None;
                        }
                    });

                    ui.collapsing("Rendering options", |ui| {
                        ui.collapsing("Reset to default", |ui| {
                            if ui.button("Load default render settings").clicked() {
                                self.config = RenderConfig::default();
                            }
                        });

                        ui.add(
                            egui::Slider::new(&mut self.config.image_width, 1..=1000)
                                .suffix("px")
                                .text("Image width"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.image_height, 1..=500)
                                .suffix("px")
                                .text("Image height"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.samples_per_pixel, 1..=200)
                                .text("Samples per pixel"),
                        );
                        ui.end_row();

                        egui::ComboBox::from_label("Render mode")
                            .selected_text(match self.config.render_mode {
                                RayColorMode::BlockColor { .. } => "Block color",
                                RayColorMode::ShadeNormal => "Normals",
                                RayColorMode::Depth { .. } => "Depth test",
                                RayColorMode::Material { .. } => "Material",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.config.render_mode,
                                    RayColorMode::BlockColor {
                                        color: Color::new(255.0, 0.0, 0.0),
                                    },
                                    "Block color",
                                );
                                ui.selectable_value(
                                    &mut self.config.render_mode,
                                    RayColorMode::Depth { max_t: 1.0 },
                                    "Depth test",
                                );
                                ui.selectable_value(
                                    &mut self.config.render_mode,
                                    RayColorMode::ShadeNormal,
                                    "Normals",
                                );
                                ui.selectable_value(
                                    &mut self.config.render_mode,
                                    RayColorMode::Material { depth: 50 },
                                    "Material",
                                );
                            });
                        ui.end_row();

                        let sub_heading = "Mode settings";
                        match self.config.render_mode {
                            RayColorMode::BlockColor { ref mut color } => {
                                ui.collapsing(sub_heading, |ui| {
                                    vec3_editor(ui, "Color", color);
                                });
                            }
                            RayColorMode::ShadeNormal => (),
                            RayColorMode::Depth { ref mut max_t } => {
                                ui.collapsing(sub_heading, |ui| {
                                    ui.add(egui::Slider::new(max_t, 0.0..=20.0).text("Distance"));
                                });
                            }
                            RayColorMode::Material { ref mut depth } => {
                                ui.collapsing(sub_heading, |ui| {
                                    ui.add(
                                        egui::Slider::new(depth, 1..=100)
                                            .text("Depth")
                                            .clamp_to_range(true),
                                    );
                                });
                            }
                        }
                    });

                    ui.collapsing("Camera options", |ui| {
                        let current_scene = self.config.scene;
                        let cam = self
                            .scene_to_camera
                            .entry(self.config.scene)
                            .or_insert_with(|| current_scene.default_camera_settings());
                        ui.collapsing("Reset to default", |ui| {
                            if ui.button("Load default camera settings").clicked() {
                                *cam = current_scene.default_camera_settings();
                            }
                        });

                        vec3_editor(ui, "Look from", &mut cam.look_from);
                        ui.end_row();

                        vec3_editor(ui, "Look at", &mut cam.look_at);
                        ui.end_row();

                        vec3_editor(ui, "Up direction", &mut cam.vup);
                        ui.end_row();

                        ui.add(
                            egui::widgets::Slider::new(&mut cam.vfov, 10.0..=30.0)
                                .text("Vertical FoV"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::widgets::Slider::new(&mut cam.focus_dist, 0.0..=30.0)
                                .text("Focus distance"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::widgets::Slider::new(&mut cam.aperture, 0.0..=2.0)
                                .text("Aperture size"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::widgets::Slider::new(&mut cam.time0, 0.0..=(cam.time1))
                                .suffix("s")
                                .clamp_to_range(true)
                                .text("Aperture open time"),
                        );
                        ui.end_row();
                        ui.add(
                            egui::widgets::Slider::new(&mut cam.time1, cam.time0..=3.0)
                                .suffix("s")
                                .clamp_to_range(true)
                                .text("Aperture close time"),
                        );
                        ui.end_row();
                    });
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref mut data) = self.data {
                ui.add(
                    egui::ProgressBar::new(data.percent_complete())
                        .animate(!data.complete())
                        .desired_width(if self.display.display_actual_size {
                            data.last_render_width as f32
                        } else {
                            ui.available_width()
                        }),
                );

                let image_sizing = if self.display.display_actual_size {
                    egui::Vec2::new(
                        data.last_render_width as f32,
                        data.last_render_height as f32,
                    )
                } else {
                    let mut available = ui.available_size();
                    available.y -= 25.0;
                    available
                };

                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    if let Some(tex_id) = data.last_render_tex {
                        ui.image(tex_id, image_sizing);
                    }
                });
            }
        });
    }
}

fn vec3_editor(ui: &mut egui::Ui, label: &str, v: &mut Vec3) {
    let speed = 0.1;

    ui.horizontal(|ui| {
        ui.label("x");
        ui.add(egui::widgets::DragValue::new(&mut v.x).speed(speed));
        ui.label("y");
        ui.add(egui::widgets::DragValue::new(&mut v.y).speed(speed));
        ui.label("z");
        ui.add(egui::widgets::DragValue::new(&mut v.z).speed(speed));

        ui.label(label);
    });
}
