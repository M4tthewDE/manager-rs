use egui::{CentralPanel, Color32, Context, RichText, ScrollArea};

use crate::App;

mod compose;
mod docker;
mod info;

impl App {
    pub fn ui(&self, ctx: &Context) {
        puffin::profile_function!();
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ScrollArea::vertical().id_source("ui").show(ui, |ui| {
                    ui.heading(RichText::new("Server manager").color(Color32::WHITE));
                    ui.add_space(10.0);

                    info::info(ui, &self.state.info);
                    ui.add_space(10.0);

                    self.docker(ui);
                    ui.add_space(10.0);

                    self.compose(ui);
                });
            });
        });
    }
}
