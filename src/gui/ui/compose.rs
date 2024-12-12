use egui::{Color32, RichText, Ui};

use crate::state::compose::{ComposeFileDiff, DiffResult};

pub fn compose(ui: &mut Ui, diffs: &[ComposeFileDiff]) {
    ui.heading(RichText::new("Compose").color(Color32::WHITE));

    ui.group(|ui| {
        ui.vertical(|ui| {
            for f in diffs {
                file(ui, f);
            }
        });
    });
}

pub fn file(ui: &mut Ui, diff: &ComposeFileDiff) {
    ui.horizontal(|ui| {
        ui.label(&diff.name);
        match diff.result {
            DiffResult::New => ui.label(RichText::new("New").color(Color32::GREEN)),
            DiffResult::Same => ui.label(RichText::new("Unchanged").color(Color32::GRAY)),
            DiffResult::Modified => ui.label(RichText::new("Modified").color(Color32::YELLOW)),
        }
    });
}
