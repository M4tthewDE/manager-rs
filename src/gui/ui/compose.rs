use std::sync::mpsc::Sender;

use anyhow::{Context, Result};
use egui::{Color32, RichText, Ui};
use tracing::error;

use crate::config::Config;
use crate::proto::ComposeFile;
use crate::state::compose::{ComposeFileDiff, DiffResult};
use crate::state::State;
use crate::update::StateChangeMessage;
use crate::{client, App};

impl App {
    pub fn compose(&self, ui: &mut Ui) {
        puffin::profile_function!();

        ui.horizontal(|ui| {
            ui.heading(RichText::new("Compose").color(Color32::WHITE));
            if ui.button("⟳").clicked() {
                let config = self.config.clone();
                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    if let Err(err) = update_compose_diffs(config, tx).await {
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

async fn update_compose_diffs(config: Config, tx: Sender<StateChangeMessage>) -> Result<()> {
    let mut files = Vec::new();

    for dir_entry in config.docker_compose_path.read_dir()? {
        let dir_entry = dir_entry?;
        files.push(ComposeFile {
            name: dir_entry
                .file_name()
                .to_str()
                .context("invalid file name {p:?}")?
                .to_string(),
            content: std::fs::read_to_string(dir_entry.path())?,
        });
    }

    let diffs = crate::client::compose::diff_files(files, config.server_address).await?;

    Ok(tx.send(Box::new(move |state: &mut State| {
        state.compose_file_diffs = diffs;
    }))?)
}
