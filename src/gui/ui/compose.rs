use std::path::PathBuf;

use egui::{Color32, RichText, Ui};
use tracing::error;

use crate::state::compose::{ComposeFileDiff, DiffResult};
use crate::{client, update, App};

impl App {
    pub fn compose(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.horizontal(|ui| {
            ui.heading(RichText::new("Compose").color(Color32::WHITE));
            if ui.button("⟳").clicked() {
                let config = self.config.clone();
                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    if let Err(err) = update::update_compose_diffs(config, tx).await {
                        error!("Update compose diff error: {err:?}");
                    }
                });
            }
        });

        if self.state.compose_file_diffs.is_empty() {
            return;
        }

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
            ui.label(format!("{:?}", &diff.path));
            match diff.result {
                DiffResult::New => ui.label(RichText::new("New").color(Color32::GREEN)),
                DiffResult::Same => ui.label(RichText::new("Unchanged").color(Color32::GRAY)),
                DiffResult::Modified => ui.label(RichText::new("Modified").color(Color32::YELLOW)),
                DiffResult::Removed => ui.label(RichText::new("Removed").color(Color32::RED)),
            };

            if matches!(diff.result, DiffResult::Same) {
                if ui.button("Deploy").clicked() {
                    self.deploy(diff.path.clone());
                }
                return;
            }

            if ui.button("Push").clicked() {
                self.push(diff);
            }
        });
    }

    fn push(&self, diff: &ComposeFileDiff) {
        let config = self.config.clone();
        let d = diff.clone();
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            if let Err(err) = client::compose::push_file(config.server_address.clone(), d).await {
                error!("{err:?}");
            }

            if let Err(err) = update::update_compose_diffs(config, tx).await {
                error!("Update compose diff error: {err:?}");
            }
        });
    }

    fn deploy(&self, path: PathBuf) {
        let config = self.config.clone();

        self.rt.spawn(async move {
            if let Err(err) = client::compose::deploy(config.server_address.clone(), path).await {
                error!("{err:?}");
            }
        });
    }
}
