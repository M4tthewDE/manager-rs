use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};
use tracing::error;

use anyhow::Result;
use state::{State, StateChangeMessage};
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

            ui.heading("Server manager");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Total");
                    ui.label(&self.state.memory.total);
                });

                ui.horizontal(|ui| {
                    ui.label("Free");
                    ui.label(&self.state.memory.free);
                });

                ui.horizontal(|ui| {
                    ui.label("Available");
                    ui.label(&self.state.memory.available);
                });

                for container in &self.state.containers {
                    ui.horizontal(|ui| {
                        ui.label(&container.name);
                        ui.label(&container.image);
                        ui.label(&container.status);
                    });
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
            last_update: Instant::now(),
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
}
