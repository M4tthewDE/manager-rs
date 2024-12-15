use egui::{Color32, RichText, Ui};
use tracing::error;

use crate::{client, App};
use lib::state::compose::{ComposeFileDiff, DiffResult};

impl App {
    pub fn compose(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.heading(RichText::new("Compose").color(Color32::WHITE));

        ui.group(|ui| {
            ui.vertical(|ui| {
                for f in &self.state.compose_file_diffs {
                    self.file(ui, f);
                }
            });
        });
    }

    pub fn file(&self, ui: &mut Ui, diff: &ComposeFileDiff) {
        puffin::profile_function!();

        ui.horizontal(|ui| {
            ui.label(&diff.name);
            match diff.result {
                DiffResult::New => ui.label(RichText::new("New").color(Color32::GREEN)),
                DiffResult::Same => ui.label(RichText::new("Unchanged").color(Color32::GRAY)),
                DiffResult::Modified => ui.label(RichText::new("Modified").color(Color32::YELLOW)),
                DiffResult::Removed => ui.label(RichText::new("Removed").color(Color32::RED)),
            };

            if matches!(diff.result, DiffResult::Same) {
                return;
            }

            if ui.button("Push").clicked() {
                self.push(diff);
            }
        });
    }

    fn push(&self, diff: &ComposeFileDiff) {
        let server_address = self.config.clone().server_address;
        let d = diff.clone();

        self.rt.spawn(async move {
            if let Err(err) = client::compose::push_file(server_address, d).await {
                error!("{err:?}");
            }
        });
    }
}
