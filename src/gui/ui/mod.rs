use egui::{CentralPanel, Color32, Context, RichText};

use crate::App;

mod compose;
mod docker;
mod info;

impl App {
    pub fn ui(&self, ctx: &Context) {
        puffin::profile_function!();
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new("Server manager").color(Color32::WHITE));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    info::info(ui, &self.state.info);
                    info::memory(ui, &self.state.info.memory);
                    info::cpus(ui, &self.state.info.cpus);
                });
                ui.add_space(10.0);

                info::disks(ui, &self.state.info.disks);
                ui.add_space(10.0);

                self.docker(ui);
                ui.add_space(10.0);

                self.compose(ui);
            });
        });
    }
}
