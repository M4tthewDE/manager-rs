use egui::{Color32, RichText, ScrollArea};
use std::env;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};
use tracing::error;

use anyhow::Result;
use serde::Deserialize;
use state::{State, StateChangeMessage};
use std::path::PathBuf;
use tokio::runtime;

mod state;
mod ui;

#[derive(Deserialize, Clone)]
struct Config {
    update_interval: u64,
}

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let config: Config = toml::from_str(
        &std::fs::read_to_string(PathBuf::from("config.toml"))
            .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?,
    )
    .map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Server manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(config)?))),
    )
}

struct App {
    config: Config,
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
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui::info::info(ui, &self.state.info);
                    ui::info::memory(ui, &self.state.info.memory);
                    ui::info::cpus(ui, &self.state.info.cpus);
                });
                ui.add_space(10.0);

                ui::info::disks(ui, &self.state.info.disks);
                ui.add_space(10.0);

                ui.heading(RichText::new("Docker").color(Color32::WHITE));
                ScrollArea::vertical().id_source("docker").show(ui, |ui| {
                    for c in &self.state.containers {
                        ui::docker::container(ui, c, &self.tx, &self.rt);
                    }
                });
            });

            if self.profiler {
                self.profiler = puffin_egui::profiler_window(ctx);
            }
        });
    }
}

impl App {
    fn new(config: Config) -> Result<Self> {
        let profiler = env::var("PROFILING").is_ok();
        if profiler {
            puffin::set_scopes_on(true);
        }

        let (tx, rx) = mpsc::channel();

        Ok(Self {
            config: config.clone(),
            rt: runtime::Builder::new_multi_thread().enable_all().build()?,
            last_update: Instant::now() - Duration::from_millis(config.update_interval + 1000),
            state: State::default(),
            profiler,
            tx,
            rx,
        })
    }

    fn update_state(&mut self) -> Result<()> {
        puffin::profile_function!();
        if self.last_update.elapsed().as_millis() > self.config.update_interval.into() {
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
}
