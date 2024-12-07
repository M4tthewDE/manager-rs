use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Instant,
};

use anyhow::Result;
use docker::{docker_client::DockerClient, Container, Empty};
use memory::Memory;
use memory_proto::{memory_client::MemoryClient, MemoryReply};
use tokio::runtime;

mod memory;

pub mod docker {
    tonic::include_proto!("docker");
}

pub mod memory_proto {
    tonic::include_proto!("memory");
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

type StateChangeMessage = Box<dyn FnMut(&mut State) + Send>;

struct App {
    rt: runtime::Runtime,

    state: State,
    last_update: Instant,

    tx: Sender<StateChangeMessage>,
    rx: Receiver<StateChangeMessage>,
}

#[derive(Default)]
struct State {
    containers: Vec<Container>,
    memory: Memory,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_state().unwrap();

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
            state: State::default(),
            tx,
            rx,
        })
    }

    fn update_state(&mut self) -> Result<()> {
        if self.last_update.elapsed().as_secs() > 2 {
            let tx = self.tx.clone();

            self.rt.spawn(async move {
                let containers = get_containers().await.unwrap();
                tx.send(Box::new(move |state: &mut State| {
                    state.containers = containers.clone()
                }))
                .unwrap();

                let memory_reply = get_memory().await.unwrap();
                tx.send(Box::new(move |state: &mut State| {
                    state.memory = Memory::new(&memory_reply);
                }))
                .unwrap();
            });

            self.last_update = Instant::now();
        }

        if let Ok(mut state_change_msg) = self.rx.try_recv() {
            state_change_msg(&mut self.state);
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

async fn get_memory() -> Result<MemoryReply> {
    let mut client = MemoryClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(memory_proto::Empty {});
    let response = client.get_memory(request).await?;
    Ok(*response.get_ref())
}
