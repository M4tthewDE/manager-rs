use std::sync::mpsc::Sender;

use egui::{CentralPanel, Color32, Context, RichText};
use tokio::runtime::Runtime;

use crate::state::{State, StateChangeMessage};

pub mod docker;
pub mod info;

pub fn ui(ctx: &Context, state: &State, tx: &Sender<StateChangeMessage>, rt: &Runtime) {
    CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Server manager").color(Color32::WHITE));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                info::info(ui, &state.info);
                info::memory(ui, &state.info.memory);
                info::cpus(ui, &state.info.cpus);
            });
            ui.add_space(10.0);

            info::disks(ui, &state.info.disks);
            ui.add_space(10.0);

            docker::docker(ui, &state.containers, tx, rt);
        });
    });
}
