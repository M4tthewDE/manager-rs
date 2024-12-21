use egui::{CentralPanel, Color32, Context, RichText, ScrollArea, SidePanel};

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

    pub fn log_panel(&self, ctx: &Context) {
        SidePanel::right("log_panel")
            .min_width(600.0)
            .show(ctx, |ui| {
                ui.heading(RichText::new("Server logs").color(Color32::WHITE));
                ScrollArea::both()
                    .id_source("log_scroll_area")
                    .show(ui, |ui| {
                        for line in &self.state.server_log.logs {
                            let level_text = match line.level {
                                crate::state::log::LogLevel::Trace => {
                                    RichText::new("Trace").color(Color32::WHITE)
                                }
                                crate::state::log::LogLevel::Debug => {
                                    RichText::new("Debug").color(Color32::BLUE)
                                }
                                crate::state::log::LogLevel::Info => {
                                    RichText::new("Info").color(Color32::GREEN)
                                }
                                crate::state::log::LogLevel::Warn => {
                                    RichText::new("Warn").color(Color32::YELLOW)
                                }
                                crate::state::log::LogLevel::Error => {
                                    RichText::new("Error").color(Color32::RED)
                                }
                            };

                            ui.horizontal(|ui| {
                                ui.label(level_text);
                                ui.label(&line.text);
                            });
                        }
                    });
            });
    }
}
