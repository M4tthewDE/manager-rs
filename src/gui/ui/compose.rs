use std::sync::mpsc::Sender;

use egui::{Color32, RichText, Ui};
use tokio::runtime::Runtime;
use tracing::error;

use crate::{
    config::Config,
    state::{
        self,
        compose::{ComposeFileDiff, DiffResult},
        StateChangeMessage,
    },
};

pub fn compose(
    ui: &mut Ui,
    diffs: &[ComposeFileDiff],
    tx: &Sender<StateChangeMessage>,
    rt: &Runtime,
    config: Config,
) {
    puffin::profile_function!();

    ui.heading(RichText::new("Compose").color(Color32::WHITE));

    ui.group(|ui| {
        ui.vertical(|ui| {
            for f in diffs {
                file(ui, f, tx, rt, config.clone());
            }
        });
    });
}

pub fn file(
    ui: &mut Ui,
    diff: &ComposeFileDiff,
    tx: &Sender<StateChangeMessage>,
    rt: &Runtime,
    config: Config,
) {
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
            let tx = tx.clone();

            let d = diff.clone();
            rt.spawn(async move {
                if let Err(err) = state::compose::push_file(config.clone().server_address, d).await
                {
                    error!("{err:?}");
                }

                if let Err(err) = state::update(tx, config).await {
                    error!("{err:?}");
                }
            });
        }
    });
}
