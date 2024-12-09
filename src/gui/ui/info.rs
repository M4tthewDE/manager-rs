use egui::{Color32, RichText, Ui};

use crate::state::info::{cpu::Cpu, disk::Disk, memory::Memory, Info};

pub fn disks(ui: &mut Ui, disks: &[Disk]) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("Disks").color(Color32::WHITE));
        ui.horizontal(|ui| {
            for d in disks {
                disk(ui, d);
            }
        });
    });
}

fn disk(ui: &mut Ui, disk: &Disk) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").color(Color32::WHITE));
                ui.label(&disk.name);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Kind").color(Color32::WHITE));
                ui.label(&disk.kind);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("File System").color(Color32::WHITE));
                ui.label(&disk.file_system);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Total").color(Color32::WHITE));
                ui.label(&disk.total_space);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Available").color(Color32::WHITE));
                ui.label(&disk.available_space);
            });
        });
    });
}

pub fn info(ui: &mut Ui, info: &Info) {
    puffin::profile_function!();

    ui.vertical(|ui| {
        ui.heading(RichText::new("Info").color(Color32::WHITE));
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").color(Color32::WHITE));
                ui.label(&info.name);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Kernel version").color(Color32::WHITE));
                ui.label(&info.kernel_version);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("OS version").color(Color32::WHITE));
                ui.label(&info.os_version);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Host name").color(Color32::WHITE));
                ui.label(&info.host_name);
            });
        });
    });
}

pub fn memory(ui: &mut Ui, memory: &Memory) {
    puffin::profile_function!();

    ui.vertical(|ui| {
        ui.heading(RichText::new("Memory").color(Color32::WHITE));
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Total").color(Color32::WHITE));
                ui.label(&memory.total);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Free").color(Color32::WHITE));
                ui.label(&memory.free);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Available").color(Color32::WHITE));
                ui.label(&memory.available);
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Used").color(Color32::WHITE));
                ui.label(&memory.used);
            });
        });
    });
}

pub fn cpus(ui: &mut Ui, cpus: &[Cpu]) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("CPU").color(Color32::WHITE));
        ui.group(|ui| {
            egui::Grid::new("cpus").show(ui, |ui| {
                for (i, c) in cpus.iter().enumerate() {
                    if i % 4 == 0 && i != 0 {
                        ui.end_row();
                    }
                    cpu(ui, c);
                }
            });
        });
    });
}

fn cpu(ui: &mut Ui, cpu: &Cpu) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(&cpu.name).color(Color32::WHITE));
        let color = if cpu.usage < 50.0 {
            Color32::GREEN
        } else if cpu.usage < 75.0 {
            Color32::YELLOW
        } else {
            Color32::RED
        };
        ui.label(RichText::new(format!("{:.2}%", cpu.usage)).color(color));
        ui.label(&cpu.frequency);
    });
}
