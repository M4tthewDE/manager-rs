use std::sync::mpsc::Sender;

use egui::{CollapsingHeader, Color32, RichText, ScrollArea, TextStyle, Ui};
use tokio::runtime::Runtime;
use tracing::error;

use crate::{
    config::Config,
    state::{
        self,
        docker::{Container, DockerState, Port, Version},
        StateChangeMessage,
    },
};

pub fn docker(
    ui: &mut Ui,
    docker_state: &DockerState,
    tx: &Sender<StateChangeMessage>,
    rt: &Runtime,
    config: Config,
) {
    puffin::profile_function!();

    ui.heading(RichText::new("Docker").color(Color32::WHITE));
    version(ui, &docker_state.version);
    ScrollArea::vertical().id_source("docker").show(ui, |ui| {
        for c in &docker_state.containers {
            container(ui, c, tx, rt, config.clone());
        }
    });
}

fn version(ui: &mut Ui, version: &Version) {
    ui.horizontal(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Version").color(Color32::WHITE));
            ui.label(&version.version);
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Api version").color(Color32::WHITE));
            ui.label(&version.api_version);
        });
    });
}

fn container(
    ui: &mut Ui,
    container: &Container,
    tx: &Sender<StateChangeMessage>,
    rt: &Runtime,
    config: Config,
) {
    puffin::profile_function!();

    ui.group(|ui| {
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
            ui.horizontal(|ui| {
                ports(ui, &container.ports);
            });
        });

        logs(ui, container);
        docker_actions(ui, &container.id, tx, rt, config);
    });
}

fn ports(ui: &mut Ui, ports: &[Port]) {
    puffin::profile_function!();

    ui.label(RichText::new("Ports").color(Color32::WHITE));
    ui.vertical(|ui| {
        for p in ports {
            ui.label(format!(
                "{}->{}/{}",
                p.public_port, p.private_port, p.port_type
            ));
        }
    });
}

fn logs(ui: &mut Ui, container: &Container) {
    puffin::profile_function!();

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
}

fn docker_actions(
    ui: &mut Ui,
    id: &str,
    tx: &Sender<StateChangeMessage>,
    rt: &Runtime,
    config: Config,
) {
    puffin::profile_function!();

    ui.horizontal(|ui| {
        if ui.button("Start").clicked() {
            start_container(id.to_owned(), tx, rt, config.clone())
        }
        if ui.button("Stop").clicked() {
            stop_container(id.to_owned(), tx, rt, config.clone())
        }
        if ui.button("Remove").clicked() {
            remove_container(id.to_owned(), tx, rt, config.clone())
        }
    });
}

fn start_container(id: String, tx: &Sender<StateChangeMessage>, rt: &Runtime, config: Config) {
    puffin::profile_function!();

    let tx = tx.clone();

    rt.spawn(async move {
        if let Err(err) = state::start_container(id, config.server_address.clone()).await {
            error!("{err:?}");
        }

        if let Err(err) = state::update(tx, config).await {
            error!("{err:?}");
        }
    });
}

fn stop_container(id: String, tx: &Sender<StateChangeMessage>, rt: &Runtime, config: Config) {
    puffin::profile_function!();

    let tx = tx.clone();

    rt.spawn(async move {
        if let Err(err) = state::stop_container(id, config.server_address.clone()).await {
            error!("{err:?}");
        }

        if let Err(err) = state::update(tx, config).await {
            error!("{err:?}");
        }
    });
}

fn remove_container(id: String, tx: &Sender<StateChangeMessage>, rt: &Runtime, config: Config) {
    puffin::profile_function!();

    let tx = tx.clone();

    rt.spawn(async move {
        if let Err(err) = state::remove_container(id, config.server_address.clone()).await {
            error!("{err:?}");
        }

        if let Err(err) = state::update(tx, config).await {
            error!("{err:?}");
        }
    });
}
