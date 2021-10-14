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
}

impl UiData {
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
    data: UiData,
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
            data: Default::default(),
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
        assert_eq!(line_pixels.len(), self.config.image_width);
        assert!(self.data.last_render_lines_received < self.data.last_render_height);
        self.data.last_render_lines_received += 1;

        // let mut image_buffer =
        //     vec![RGB8 { r: 0, g: 0, b: 0 }; self.config.image_pixel_count()];

        // update the image buffer
        let line_num = self.config.image_height - line_num - 1;
        let offset_start = line_num as usize * self.config.image_width;
        let offset_end = offset_start + self.config.image_width;
        self.data.last_render_pixels[offset_start..offset_end]
            .copy_from_slice(line_pixels.as_slice());

        // //terminal progress indicator state
        // let progress_indicator_width = 100i32;
        // let progress_indicator_height =
        //     (progress_indicator_width as f64 / self.config.aspect_ratio / 2.0) as i32;
        // let width_incr = self.config.image_width as i32 / progress_indicator_width;
        // let height_incr = self.config.image_height as i32 / progress_indicator_height;
        // let mut progress_lines_written = 0;

        // // render the terminal progress indicator display
        // // we only want to update the terminal render-in-progress display if the line we just
        // // received is actually going to change the display
        // if self.incremental_progress_display && line_num as i32 % height_incr == 0 {
        //     if progress_lines_written > 0 {
        //         print!("{}", termion::cursor::Up(progress_lines_written));
        //         progress_lines_written = 0;
        //     }
        //     for j in 0..(self.config.image_height) {
        //         let showing_progress_for_this_line = j % height_incr as usize == 0;
        //         for i in 0..(self.config.image_width) {
        //             if showing_progress_for_this_line && i % width_incr as usize == 0 {
        //                 let c = rgb8_as_terminal_char(
        //                     image_buffer[j * self.config.image_width + i],
        //                 );
        //                 print!("{}", c);
        //             }
        //         }
        //         if showing_progress_for_this_line {
        //             println!();
        //             progress_lines_written += 1;
        //         }
        //     }
        // }

        if self.data.complete() {
            // make sure we got all the data we should have
            assert_eq!(
                self.data.last_render_pixels.len(),
                self.config.image_pixel_count()
            );

            println!(
                "Saving completed image to disk at {} in PNG format...",
                self.config.output_filename
            );
            lodepng::encode_file(
                &self.config.output_filename,
                &self.data.last_render_pixels,
                self.config.image_width,
                self.config.image_height,
                lodepng::ColorType::RGB,
                8,
            )
            .expect("Encoding result and saving to disk failed");

            println!("Done saving.");
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "All Seeing Crab"
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
                    self.data.last_render_width = image_width;
                    self.data.last_render_height = image_height;
                    self.data.last_render_pixels =
                        vec![RGB8 { r: 0, g: 0, b: 0 }; self.config.image_pixel_count()];
                    self.data.last_render_lines_received = 0;

                    if let Some(existing_tex) = self.data.last_render_tex {
                        frame.tex_allocator().free(existing_tex);
                        self.data.last_render_tex = None;
                    }
                }
                Ok(RenderResult::ImageLine {
                    line_num,
                    line_pixels,
                }) => {
                    self.store_pixel_line(line_num, line_pixels);

                    // update the texture that gets displayed in the UI
                    if let Some(existing_tex) = self.data.last_render_tex {
                        frame.tex_allocator().free(existing_tex);
                    }
                    let tex_pixels = self
                        .data
                        .last_render_pixels
                        .iter()
                        .map(|rgb| egui::Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 255))
                        .collect::<Vec<_>>();
                    self.data.last_render_tex = Some(
                        frame
                            .tex_allocator()
                            .alloc_srgba_premultiplied((400, 225), &tex_pixels),
                    );
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

            ui.horizontal(|ui| {
                ui.label("Output filename: ");
                ui.text_edit_singleline(&mut self.config.output_filename);
            });

            ui.add(
                egui::Slider::new(&mut self.config.samples_per_pixel, 0..=200)
                    .text("Samples per pixel"),
            );
            if ui.button("Render").clicked() {
                self.trigger_render();
            }
            egui::warn_if_debug_build(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ray tracing result");
            let sizing = egui::Vec2::new(400 as f32, 225 as f32);
            if let Some(tex_id) = self.data.last_render_tex {
                ui.image(tex_id, sizing);
            }
            if !self.data.complete() {
                ui.add(egui::ProgressBar::new(self.data.percent_complete()).animate(true));
            }
        });
    }
}
