use eframe::{
    egui::{self, TextureId},
    epi,
};
use rgb::RGB8;

use crate::{color::rgb8_as_terminal_char, RenderCommand, RenderConfig, RenderResult};

#[derive(Debug, Default)]
struct UiData {
    last_render_width: usize,
    last_render_height: usize,
    last_render_lines_received: usize,
    last_render_pixels: Vec<RGB8>,
    last_render_tex: Option<TextureId>,

    terminal_initial_render_done: bool,
    lines_received_since_last_terminal_render: Vec<usize>,
}

impl UiData {
    fn new(width: usize, height: usize) -> Self {
        let mut d = Self::default();
        d.last_render_width = width;
        d.last_render_height = height;
        d.last_render_pixels = vec![RGB8 { r: 0, g: 0, b: 0 }; width * height];
        d
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
        self.last_render_tex =
            Some(tex_allocator.alloc_srgba_premultiplied((400, 225), &tex_pixels));
    }

    fn clear_texture(&mut self, tex_allocator: &mut dyn eframe::epi::TextureAllocator) {
        if let Some(existing_tex) = self.last_render_tex {
            tex_allocator.free(existing_tex);
            self.last_render_tex = None;
        }
    }

    fn save_output_to_file(&self, output_filename: &str) {
        // make sure we got all the data we should have
        assert_eq!(
            self.last_render_pixels.len(),
            self.last_render_width * self.last_render_height
        );

        println!(
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

        println!("Done saving.");
    }

    fn complete(&self) -> bool {
        self.last_render_lines_received == self.last_render_height
    }

    fn percent_complete(&self) -> f32 {
        self.last_render_lines_received as f32 / self.last_render_height as f32
    }
}

#[derive(Debug)]
pub struct TemplateApp {
    config: RenderConfig,
    data: Option<UiData>,
    incremental_progress_display: bool,

    render_command_tx: flume::Sender<RenderCommand>,
    render_result_rx: flume::Receiver<RenderResult>,
}

impl TemplateApp {
    pub(crate) fn new(
        output_filename: &str,
        render_command_tx: flume::Sender<RenderCommand>,
        render_result_rx: flume::Receiver<RenderResult>,
    ) -> Self {
        let mut config = RenderConfig::default();
        config.output_filename = output_filename.to_owned();
        TemplateApp {
            config,
            data: None,
            incremental_progress_display: true,
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
                config: self.config.clone(),
            })
            .ok()
            .expect("render command send should succeed");
    }

    fn store_pixel_line(&mut self, line_num: usize, line_pixels: Vec<RGB8>) {
        let data = self
            .data
            .as_mut()
            .expect("data must be present if storing pixel line");

        assert_eq!(line_pixels.len(), data.last_render_width);
        assert!(data.last_render_lines_received < data.last_render_height);
        data.last_render_lines_received += 1;
        data.lines_received_since_last_terminal_render
            .push(line_num);

        // update the image buffer
        let line_num = self.config.image_height - line_num - 1;
        let offset_start = line_num as usize * self.config.image_width;
        let offset_end = offset_start + self.config.image_width;
        data.last_render_pixels[offset_start..offset_end].copy_from_slice(line_pixels.as_slice());

        if data.complete() {}
    }

    fn render_terminal_progress_indicator(&mut self) {
        let data = self.data.as_mut().unwrap();

        let progress_indicator_width = 100i32;
        let progress_indicator_height =
            (progress_indicator_width as f64 / self.config.aspect_ratio() / 2.0) as i32;
        let width_incr = data.last_render_width as i32 / progress_indicator_width;
        let height_incr = data.last_render_height as i32 / progress_indicator_height;

        // We only render some rows and columns of pixels, and terminals can be slow, so
        // we only want to update the terminal render-in-progress display if the lines we've
        // received since last render would actually change the displayed output.
        let should_rerender = self.incremental_progress_display
            && false // TODO this is currently broken - remove this line and fix the bug
            && data
                .lines_received_since_last_terminal_render
                .iter()
                .any(|line_num| line_num % height_incr as usize == 0);
        if should_rerender {
            if data.terminal_initial_render_done {
                print!("{}", termion::cursor::Up(data.last_render_height as u16));
            }
            for j in 0..(data.last_render_height) {
                let showing_progress_for_this_line = j % height_incr as usize == 0;
                if showing_progress_for_this_line {
                    for i in 0..(data.last_render_width) {
                        if i % width_incr as usize == 0 {
                            let c = rgb8_as_terminal_char(
                                data.last_render_pixels[j * data.last_render_width + i],
                            );
                            print!("{}", c);
                        }
                    }
                    println!();
                }
            }

            data.lines_received_since_last_terminal_render.clear();
            data.terminal_initial_render_done = true;
        }
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
                    self.data
                        .as_mut()
                        .map(|d| d.clear_texture(frame.tex_allocator()));
                    self.data = Some(UiData::new(image_width, image_height));
                }
                Ok(RenderResult::ImageLine {
                    line_num,
                    line_pixels,
                }) => {
                    self.store_pixel_line(line_num, line_pixels);
                    self.render_terminal_progress_indicator();

                    let data = self
                        .data
                        .as_mut()
                        .expect("ui data must be present after storing pixels");
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("config_panel").show(ctx, |ui| {
            ui.heading("Config");

            ui.add(egui::Slider::new(&mut self.config.image_width, 1..=1000).text("Image width"));
            ui.add(egui::Slider::new(&mut self.config.image_height, 1..=500).text("Image height"));

            ui.add(
                egui::Slider::new(&mut self.config.samples_per_pixel, 1..=200)
                    .text("Samples per pixel"),
            );

            ui.checkbox(&mut self.config.generate_random_scene, "Random scene");

            ui.horizontal(|ui| {
                ui.label("Output filename: ");
                ui.text_edit_singleline(&mut self.config.output_filename);
            });

            if ui.button("Render").clicked() {
                self.trigger_render();
            }
            egui::warn_if_debug_build(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ray tracing result");
            if let Some(ref data) = self.data {
                let sizing = egui::Vec2::new(
                    data.last_render_width as f32,
                    data.last_render_height as f32,
                );
                if let Some(tex_id) = data.last_render_tex {
                    ui.image(tex_id, sizing);
                }
                if !data.complete() {
                    ui.add(egui::ProgressBar::new(data.percent_complete()).animate(true));
                }
            }
        });
    }
}
