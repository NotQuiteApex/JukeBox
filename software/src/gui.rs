use std::ops::Add;
use std::thread;
use std::time::Instant;
use std::{sync::mpsc::channel, time::Duration};

use eframe::egui;
use egui::{Align, Color32, RichText};

use crate::serial::{serial_get_device, serial_task, SerialCommand, SerialEvent};
use crate::system::{PCSystem, SystemReport};

enum ConnectionStatus {
    Connected,
    NotConnected,
    LostConnection,
}

pub fn basic_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_maximize_button(false)
            .with_resizable(false),
        ..Default::default()
    };

    let (serialevent_tx, serialevent_rx) = channel::<SerialEvent>();
    let (serialcommand_tx, serialcommand_rx) = channel::<SerialCommand>();
    let (sysreport_tx1, sysreport_rx1) = channel::<SystemReport>();
    let (sysreport_tx2, sysreport_rx2) = channel::<SystemReport>();

    let (breaker_tx1, breaker_rx1) = channel::<bool>();
    let (breaker_tx2, breaker_rx2) = channel::<bool>();

    // system stats thread
    let systemstats = thread::spawn(move || {
        // TODO: handle removable PC hardware (such as external GPUs)
        let mut pcs = PCSystem::new().expect("COULD NOT MAKE PC REPORTER");
        let mut timer = Instant::now();

        loop {
            if let Ok(_) = breaker_rx1.try_recv() {
                break;
            }
            if Instant::now() < timer {
                continue;
            }
            timer = Instant::now().add(Duration::from_secs(1));

            sysreport_tx1
                .send(pcs.get_report())
                .expect("COULD NOT SEND PC REPORT 1"); // send to gui
            sysreport_tx2
                .send(pcs.get_report())
                .expect("COULD NOT SEND PC REPORT 2"); // send to serial
            pcs.update();
        }
    });

    // serial comms thread
    let serialcomms = thread::spawn(move || {
        loop {
            if let Ok(_) = breaker_rx2.try_recv() {
                break;
            }

            let f = serial_get_device();
            if let Err(_) = f {
                // log::error!("Failed to get serial device. Error: `{}`.", e);
                continue;
            }
            let mut f = f.unwrap();

            match serial_task(&mut f, &sysreport_rx1, &serialcommand_rx, &serialevent_tx) {
                Err(e) => {
                    log::warn!("Serial device error: `{}`", e);
                    if let Err(e) = serialevent_tx.send(SerialEvent::LostConnection) {
                        log::warn!("LostConnection event signal failed, reason: `{}`", e);
                    }
                }
                Ok(_) => log::info!("Serial device successfully disconnected. Looping..."),
            };
        }
    });

    let mut connection_status = ConnectionStatus::NotConnected;
    let mut sr = SystemReport::default();
    let serialcommand_tx1 = serialcommand_tx.clone();

    eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
        while let Ok(snsr) = sysreport_rx2.try_recv() {
            sr = snsr;
        }
        while let Ok(event) = serialevent_rx.try_recv() {
            match event {
                SerialEvent::Connected => connection_status = ConnectionStatus::Connected,
                SerialEvent::LostConnection => connection_status = ConnectionStatus::LostConnection,
                SerialEvent::Disconnected => connection_status = ConnectionStatus::NotConnected,
            }
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
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    match connection_status {
                        ConnectionStatus::Connected => ui.label(RichText::new("Connected.").color(Color32::from_rgb(50, 200, 50))),
                        ConnectionStatus::NotConnected => ui.label(RichText::new("Not connected.").color(Color32::from_rgb(200, 200, 50))),
                        ConnectionStatus::LostConnection => ui.label(RichText::new("Lost connection!").color(Color32::from_rgb(200, 50, 50))),
                    };
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("CPU: ");
                    ui.label("CPU Freq: ");
                    ui.label("CPU Load: ");
                    ui.label("CPU Temp: ");
                    ui.label("GPU: ");
                    ui.label("GPU Core Freq: ");
                    ui.label("GPU Core Load: ");
                    ui.label("GPU VRAM Freq: ");
                    ui.label("GPU VRAM Load: ");
                    ui.label("GPU Temp: ");
                    ui.label("Memory Used: ");
                    ui.label("Memory Total: ");
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label(format!("'{}'", sr.cpu_name));
                    ui.label(format!("{} GHz", sr.cpu_freq));
                    ui.label(format!("{} %", sr.cpu_load));
                    ui.label(format!("{} ° C", sr.cpu_temp));
                    ui.label(format!("'{}'", sr.gpu_name));
                    ui.label(format!("{} MHz", sr.gpu_core_clock));
                    ui.label(format!("{} %", sr.gpu_core_load));
                    ui.label(format!("{} MHz", sr.gpu_memory_clock));
                    ui.label(format!("{} %", sr.gpu_memory_load));
                    ui.label(format!("{} ° C", sr.gpu_temp));
                    ui.label(format!("{} GiB", sr.memory_used));
                    ui.label(format!("{} GiB", sr.memory_total));
                });
                ui.separator();
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Set RGB to red").clicked() {
                    serialcommand_tx1
                        .send(SerialCommand::TestCommand)
                        .expect("failed to send command");
                    println!("you shouldnt have done that");
                }
                if ui.button("Update JukeBox").clicked() {
                    serialcommand_tx1
                        .send(SerialCommand::UpdateDevice)
                        .expect("failed to send command");
                    println!("Updating JukeBox...");
                }
            });

            ui.separator();
        });

        // Call a new frame every frame, bypassing the limited updates.
        // NOTE: This is a bad idea, we should probably change this later and only update the window as necessary.
        ctx.request_repaint();
    })
    .expect("eframe error");

    breaker_tx1
        .send(true)
        .expect("could not send breaker 1 signal");
    breaker_tx2
        .send(true)
        .expect("could not send breaker 2 signal");

    serialcommand_tx
        .send(SerialCommand::DisconnectDevice)
        .expect("could not send disconnect signal");

    serialcomms
        .join()
        .expect("could not rejoin serialcomms thread");
    systemstats
        .join()
        .expect("could not rejoin systemstats thread");
}
