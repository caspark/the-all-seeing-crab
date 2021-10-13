use eframe::{
    egui::{self, TextureId},
    epi,
};
use rgb::RGB8;

use crate::{RenderCommand, RenderResult};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct UiSettings {
    label: String,

    #[serde(skip)]
    value: f32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

#[derive(Debug, Default)]
struct UiData {
    last_render_result: Option<Vec<RGB8>>,
    last_render_tex: Option<TextureId>,
}

#[derive(Debug)]
pub struct TemplateApp {
    settings: UiSettings,
    data: UiData,

    render_command_tx: flume::Sender<RenderCommand>,
    render_result_rx: flume::Receiver<RenderResult>,
}

impl TemplateApp {
    pub(crate) fn new(
        render_command_tx: flume::Sender<RenderCommand>,
        render_result_rx: flume::Receiver<RenderResult>,
    ) -> Self {
        TemplateApp {
            settings: Default::default(),
            data: Default::default(),
            render_command_tx,
            render_result_rx,
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
            self.settings = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.render_command_tx
            .send(RenderCommand::Render)
            .ok()
            .expect("initial render command send should succeed");
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.settings);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        match self.render_result_rx.try_recv() {
            Ok(RenderResult::Image { data }) => {
                let pixels = data
                    .iter()
                    .map(|rgb| egui::Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 255))
                    .collect::<Vec<_>>();

                // let pixels: Vec<_> = (0..(width * height))
                //     .into_iter()
                //     .map(|_| {
                //         egui::Color32::from_rgba_premultiplied(
                //             rand::random(),
                //             rand::random(),
                //             rand::random(),
                //             255,
                //         )
                //     })
                //     .collect();

                if let Some(existing_tex) = self.data.last_render_tex {
                    frame.tex_allocator().free(existing_tex);
                }

                self.data.last_render_tex = Some(
                    frame
                        .tex_allocator()
                        .alloc_srgba_premultiplied((400, 225), &pixels),
                );

                self.data.last_render_result = Some(data);
            }
            Err(flume::TryRecvError::Empty) => (),
            Err(flume::TryRecvError::Disconnected) => {
                panic!("Rendering thread seems to have exited before UI!")
            }
        };

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.settings.label);
            });

            ui.add(egui::Slider::new(&mut self.settings.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.settings.value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("egui template");
            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);

            let sizing = egui::Vec2::new(400 as f32, 225 as f32);
            match self.data.last_render_tex {
                Some(tex) => {
                    ui.heading("image goes here");
                    ui.image(tex, sizing);
                    ui.heading("image should be above");
                }
                None => {
                    ui.heading("Still rendering...");
                }
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
