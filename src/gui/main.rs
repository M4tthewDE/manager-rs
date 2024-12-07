use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Instant,
};

use anyhow::Result;
use docker::{docker_client::DockerClient, Container, Empty};
use memory::Memory;
use tokio::runtime;

mod memory;

pub mod docker {
    tonic::include_proto!("docker");
}

fn main() -> eframe::Result {
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

    containers: Vec<Container>,
    memory: Memory,

    last_update: Instant,
    tx: Sender<Vec<Container>>,
    rx: Receiver<Vec<Container>>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_state().unwrap();

            ui.heading("Server manager");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Total");
                    ui.label(&self.memory.total);
                });

                ui.horizontal(|ui| {
                    ui.label("Free");
                    ui.label(&self.memory.free);
                });

                ui.horizontal(|ui| {
                    ui.label("Available");
                    ui.label(&self.memory.available);
                });

                for container in &self.containers {
                    ui.horizontal(|ui| {
                        ui.label(container.names.first().unwrap());
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
            containers: vec![],
            memory: memory::calculate_memory()?,
            tx,
            rx,
        })
    }

    fn update_state(&mut self) -> Result<()> {
        if self.last_update.elapsed().as_secs() > 2 {
            let tx = self.tx.clone();
            self.rt.spawn(async move {
                let containers = get_containers().await.unwrap();
                tx.send(containers).unwrap();
            });

            self.memory = memory::calculate_memory()?;

            self.last_update = Instant::now();
        }

        if let Ok(containers) = self.rx.try_recv() {
            self.containers = containers;
        }

        Ok(())
    }
}

async fn get_containers() -> Result<Vec<Container>> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;
    Ok(response.get_ref().container_list.clone())
}
