use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Instant,
};

use anyhow::Result;
use docker::{docker_client::DockerClient, Container, Empty};
use tokio::runtime;

pub mod docker {
    tonic::include_proto!("docker");
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

struct App {
    rt: runtime::Runtime,
    containers: Vec<Container>,

    last_update: Instant,
    tx: Sender<Vec<Container>>,
    rx: Receiver<Vec<Container>>,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            last_update: Instant::now(),
            containers: vec![],
            tx,
            rx,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_state();

            ui.heading("My egui Application");

            ui.vertical(|ui| {
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
    fn update_state(&mut self) {
        if self.last_update.elapsed().as_secs() > 2 {
            let tx = self.tx.clone();
            self.rt.spawn(async move {
                let containers = get_containers().await.unwrap();
                tx.send(containers).unwrap();
            });

            self.last_update = Instant::now();
        }

        if let Ok(containers) = self.rx.try_recv() {
            self.containers = containers;
        }
    }
}

async fn get_containers() -> Result<Vec<Container>> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(response.get_ref().container_list.clone())
}
