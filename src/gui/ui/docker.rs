use egui::{CollapsingHeader, Color32, RichText, ScrollArea, TextStyle, Ui};
use tracing::error;

use crate::state::info::docker::{
    container::{Container, Port},
    version::Version,
};
use crate::{client, App};

impl App {
    pub fn docker(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.heading(RichText::new("Docker").color(Color32::WHITE));
        version(ui, &self.state.info.docker_info.version);
        ScrollArea::vertical().id_source("docker").show(ui, |ui| {
            for c in &self.state.info.docker_info.containers {
                self.container(ui, c);
            }
        });
    }

    fn container(&self, ui: &mut Ui, container: &Container) {
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
            self.docker_actions(ui, &container.id);
        });
    }

    fn docker_actions(&self, ui: &mut Ui, id: &str) {
        puffin::profile_function!();

        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.start_container(id.to_owned())
            }
            if ui.button("Stop").clicked() {
                self.stop_container(id.to_owned())
            }
            if ui.button("Remove").clicked() {
                self.remove_container(id.to_owned())
            }
        });
    }

    fn start_container(&self, id: String) {
        puffin::profile_function!();

        let server_address = self.config.clone().server_address;

        self.rt.spawn(async move {
            if let Err(err) = client::docker::start_container(id, server_address).await {
                error!("{err:?}");
            }
        });
    }

    fn stop_container(&self, id: String) {
        puffin::profile_function!();

        let server_address = self.config.clone().server_address;

        self.rt.spawn(async move {
            if let Err(err) = client::docker::stop_container(id, server_address).await {
                error!("{err:?}");
            }
        });
    }

    fn remove_container(&self, id: String) {
        puffin::profile_function!();

        let server_address = self.config.clone().server_address;

        self.rt.spawn(async move {
            if let Err(err) = client::docker::remove_container(id, server_address).await {
                error!("{err:?}");
            }
        });
    }
}

fn version(ui: &mut Ui, version: &Version) {
    puffin::profile_function!();

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
