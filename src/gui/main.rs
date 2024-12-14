use crate::config::Config;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};
use tracing::error;

use anyhow::Result;
use state::{State, StateChangeMessage};
use tokio::runtime;

mod config;
mod state;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::new("config.toml".into())?;

    if config.profiling {
        puffin::set_scopes_on(true);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    match eframe::run_native(
        "Server manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(config)?))),
    ) {
        Ok(_) => {}
        Err(err) => error!("{err:?}"),
    };

    Ok(())
}

struct App {
    config: Config,
    rt: runtime::Runtime,

    state: State,
    last_update: Instant,

    tx: Sender<StateChangeMessage>,
    rx: Receiver<StateChangeMessage>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        ctx.request_repaint_after(Duration::from_millis(500));

        self.update_state();
        self.change_state();

        self.ui(ctx);

        if self.config.profiling {
            puffin_egui::profiler_window(ctx);
        }
    }
}

impl App {
    fn new(config: Config) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            config: config.clone(),
            rt: runtime::Builder::new_multi_thread().enable_all().build()?,
            last_update: Instant::now() - Duration::from_millis(config.update_interval + 1000),
            state: State::default(),
            tx,
            rx,
        })
    }

    fn update_state(&mut self) {
        puffin::profile_function!();

        if self.last_update.elapsed().as_millis() > self.config.update_interval.into() {
            let tx = self.tx.clone();
            let config = self.config.clone();
            self.rt.spawn(async move {
                if let Err(err) = state::update(tx, config).await {
                    error!("Update error: {err:?}");
                }
            });

            self.last_update = Instant::now();
        }
    }

    fn change_state(&mut self) {
        puffin::profile_function!();

        if let Ok(state_change_msg) = self.rx.try_recv() {
            state_change_msg(&mut self.state);
        }
    }
}
