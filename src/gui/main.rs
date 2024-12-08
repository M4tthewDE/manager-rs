use egui::{CollapsingHeader, Color32, RichText, ScrollArea, TextStyle, Ui};
use std::env;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};
use tracing::error;

use anyhow::Result;
use state::{docker::Container, memory::Memory, State, StateChangeMessage};
use tokio::runtime;

mod state;

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Server manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()?))),
    )
}

struct App {
    rt: runtime::Runtime,
    profiler: bool,

    state: State,
    last_update: Instant,

    tx: Sender<StateChangeMessage>,
    rx: Receiver<StateChangeMessage>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_state().unwrap();
            self.change_state();

            ui.vertical(|ui| {
                ui.heading(RichText::new("Server manager").color(Color32::WHITE));

                self.memory(ui, &self.state.memory);

                ui.group(|ui| {
                    ui.heading(RichText::new("Docker container").color(Color32::WHITE));
                    ScrollArea::vertical().id_source("docker").show(ui, |ui| {
                        for c in &self.state.containers {
                            ui.separator();
                            self.container(ui, c);
                        }
                    });
                });
            });

            if self.profiler {
                self.profiler = puffin_egui::profiler_window(ctx);
            }
        });
    }
}

impl App {
    fn new() -> Result<Self> {
        let profiler = env::var("PROFILING").is_ok();
        if profiler {
            puffin::set_scopes_on(true);
        }

        let (tx, rx) = mpsc::channel();

        Ok(Self {
            rt: runtime::Builder::new_multi_thread().enable_all().build()?,
            last_update: Instant::now() - Duration::from_secs(3),
            state: State::default(),
            profiler,
            tx,
            rx,
        })
    }

    fn update_state(&mut self) -> Result<()> {
        puffin::profile_function!();
        if self.last_update.elapsed().as_secs() > 2 {
            let tx = self.tx.clone();

            self.rt.spawn(async move {
                if let Err(err) = state::update(tx).await {
                    error!("Update error: {err:?}");
                }
            });

            self.last_update = Instant::now();
        }

        Ok(())
    }

    fn change_state(&mut self) {
        puffin::profile_function!();
        if let Ok(state_change_msg) = self.rx.try_recv() {
            state_change_msg(&mut self.state);
        }
    }

    fn memory(&self, ui: &mut Ui, memory: &Memory) {
        puffin::profile_function!();

        ui.group(|ui| {
            ui.heading(RichText::new("Memory").color(Color32::WHITE));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Total").color(Color32::WHITE));
                ui.label(&memory.total);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Free").color(Color32::WHITE));
                ui.label(&memory.free);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Available").color(Color32::WHITE));
                ui.label(&memory.available);
            });
        });
    }

    fn container(&self, ui: &mut Ui, container: &Container) {
        puffin::profile_function!();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Id").color(Color32::WHITE));
                ui.label(&container.id);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").color(Color32::WHITE));
                ui.label(&container.name);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Image").color(Color32::WHITE));
                ui.label(&container.image);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Status").color(Color32::WHITE));
                ui.label(&container.status);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Created").color(Color32::WHITE));
                ui.label(&container.created);
            });
        });

        CollapsingHeader::new(RichText::new("Logs").color(Color32::WHITE))
            .id_source(format!("{}-header", &container.id))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_source(container.id.clone())
                    .max_height(400.0)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Monospace),
                        container.logs.len(),
                        |ui, row_range| {
                            for line in &container.logs[row_range.start..row_range.end] {
                                ui.label(RichText::new(line).monospace());
                            }
                        },
                    );
            });

        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.start_container(container.id.clone())
            }
            if ui.button("Stop").clicked() {
                self.stop_container(container.id.clone())
            }
            if ui.button("Remove").clicked() {
                self.remove_container(container.id.clone())
            }
        });
    }

    fn start_container(&self, id: String) {
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            if let Err(err) = state::start_container(id).await {
                error!("{err:?}");
            }

            if let Err(err) = state::update(tx).await {
                error!("{err:?}");
            }
        });
    }

    fn stop_container(&self, id: String) {
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            if let Err(err) = state::stop_container(id).await {
                error!("{err:?}");
            }

            if let Err(err) = state::update(tx).await {
                error!("{err:?}");
            }
        });
    }

    fn remove_container(&self, id: String) {
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            if let Err(err) = state::remove_container(id).await {
                error!("{err:?}");
            }

            if let Err(err) = state::update(tx).await {
                error!("{err:?}");
            }
        });
    }
}
