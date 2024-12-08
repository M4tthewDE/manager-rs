use egui::{Color32, RichText, Ui};
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

    state: State,
    last_update: Instant,

    tx: Sender<StateChangeMessage>,
    rx: Receiver<StateChangeMessage>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));

        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_state().unwrap();
            self.change_state();

            ui.vertical(|ui| {
                ui.heading(RichText::new("Server manager").color(Color32::WHITE));
                ui.separator();

                ui.heading(RichText::new("Memory").color(Color32::WHITE));
                self.memory(ui, &self.state.memory);
                ui.separator();

                ui.heading(RichText::new("Docker").color(Color32::WHITE));
                for c in &self.state.containers {
                    self.container(ui, c);
                    ui.separator();
                }
            });
        });
    }
}

impl App {
    fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            rt: runtime::Builder::new_multi_thread().enable_all().build()?,
            last_update: Instant::now() - Duration::from_secs(3),
            state: State::default(),
            tx,
            rx,
        })
    }

    fn update_state(&mut self) -> Result<()> {
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
        if let Ok(state_change_msg) = self.rx.try_recv() {
            state_change_msg(&mut self.state);
        }
    }

    fn memory(&self, ui: &mut Ui, memory: &Memory) {
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
    }

    fn container(&self, ui: &mut Ui, container: &Container) {
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
            if ui.button("Remove").clicked() {
                self.remove_container(container.id.clone())
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
