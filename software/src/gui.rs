use std::{sync::mpsc::channel, time::Duration};
use std::thread::{self, sleep};

use eframe::egui;
use egui::{Align, Color32, Rect, RichText};

use crate::system::{PCSystem, SystemReport};

pub enum G2SCommands {
    UpdateDevice,
}

pub fn basic_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_maximize_button(false)
            .with_resizable(false),
        ..Default::default()
    };

    let (tx1, rx1) = channel::<G2SCommands>();
    let (tx2, rx2) = channel::<SystemReport>();

    thread::spawn(move || {
        let mut pcs = PCSystem::new().expect("COULD NOT MAKE PC REPORTER");

        loop {
            sleep(Duration::from_secs(1));
            pcs.update();
            tx2.send(pcs.get_report()).expect("COULD NOT SEND PC REPORT");
        }
    });

    let mut sr = SystemReport::default();

    eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
        let nsr = rx2.recv_timeout(Duration::from_secs(0));
        if let Ok(snsr) = nsr {
            sr = snsr;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("JukeBox Desktop")
                        .heading()
                        .strong()
                        .color(Color32::from_rgb(255, 200, 100)),
                );
                ui.label(format!(" - v{}", env!("CARGO_PKG_VERSION")));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // ui.separator();
                    ui.label("CPU: ");
                    ui.label("CPU Freq: ");
                    ui.label("CPU Load: ");
                    ui.label("CPU Temp: ");
                    // ui.separator();
                    ui.label("GPU: ");
                    ui.label("GPU Core Freq: ");
                    ui.label("GPU Core Load: ");
                    ui.label("GPU VRAM Freq: ");
                    ui.label("GPU VRAM Load: ");
                    ui.label("GPU Temp: ");
                    // ui.separator();
                    ui.label("Memory Used: ");
                    ui.label("Memory Total: ");
                    // ui.separator();
                });
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    // ui.separator();
                    ui.label(format!("{}", sr.cpu_name));
                    ui.label(format!("{}", sr.cpu_freq));
                    ui.label(format!("{}", sr.cpu_load));
                    ui.label(format!("{}", sr.cpu_temp));
                    // ui.separator();
                    ui.label(format!("{}", sr.gpu_name));
                    ui.label(format!("{}", sr.gpu_core_clock));
                    ui.label(format!("{}", sr.gpu_core_load));
                    ui.label(format!("{}", sr.gpu_memory_clock));
                    ui.label(format!("{}", sr.gpu_memory_load));
                    ui.label(format!("{}", sr.gpu_temp));
                    // ui.separator();
                    ui.label(format!("{}", sr.memory_used));
                    ui.label(format!("{}", sr.memory_total));
                    // ui.separator();
                });
            });

            ui.separator();

            if ui.button("Set RGB to red").clicked() {
                println!("you shouldnt have done that");
            }
            if ui.button("Update JukeBox").clicked() {
                println!("Updating JukeBox...");
            }

            ui.separator();

        });

        // Call a new frame every frame, bypassing the limited updates.
        // NOTE: This is a bad idea, we should probably change this later.
        ctx.request_repaint();
    })
    .expect("eframe error");
}
