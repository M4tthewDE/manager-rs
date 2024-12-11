use egui::{Color32, RichText, Ui};

use crate::state::compose::ComposeFile;

pub fn compose(ui: &mut Ui, files: &[ComposeFile]) {
    ui.heading(RichText::new("Compose").color(Color32::WHITE));

    ui.group(|ui| {
        for f in files {
            file(ui, f);
        }
    });
}

pub fn file(ui: &mut Ui, file: &ComposeFile) {
    ui.label(format!("{:?}", file.name));
}
