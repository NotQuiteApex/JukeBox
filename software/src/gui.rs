use std::ops::Add;
use std::thread;
use std::time::Instant;
use std::{sync::mpsc::channel, time::Duration};

use eframe::egui;
use egui::{Align, Color32, RichText, Vec2};

use crate::serial::{serial_get_device, serial_task, SerialCommand, SerialEvent};
use crate::system::{PCSystem, SystemReport};

#[derive(PartialEq)]
enum ConnectionStatus {
    Connected,
    NotConnected,
    LostConnection,
}

#[derive(PartialEq)]
enum GuiTab {
    Keyboard,
    System,
    Miscellaneous,
}

pub fn basic_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("JukeBox Desktop")
            .with_inner_size([480.0, 320.0])
            .with_maximize_button(false)
            .with_resizable(false)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/applogo.png")[..])
                    .unwrap(),
            ),
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

    let mut gui_tab = GuiTab::Keyboard;

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

        ctx.send_viewport_cmd(egui::ViewportCommand::Title("JUKEBOX 3000".to_owned()));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("JukeBox Desktop")
                        .heading()
                        .strong()
                        .color(Color32::from_rgb(255, 200, 100)),
                );
                let version = env!("CARGO_PKG_VERSION");
                ui.label(format!(" - v{}", version));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let res = match connection_status {
                        ConnectionStatus::Connected => {
                            ("Connected.", Color32::from_rgb(50, 200, 50))
                        }
                        ConnectionStatus::NotConnected => {
                            ("Not connected.", Color32::from_rgb(200, 200, 50))
                        }
                        ConnectionStatus::LostConnection => {
                            ("Lost connection!", Color32::from_rgb(200, 50, 50))
                        }
                    };

                    ui.label(RichText::new(res.0).color(res.1));

                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                        "JukeBox Desktop - v{} - {}",
                        version, res.0
                    )));
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut gui_tab, GuiTab::Keyboard, "Keyboard");
                ui.selectable_value(&mut gui_tab, GuiTab::System, "System");
                ui.selectable_value(&mut gui_tab, GuiTab::Miscellaneous, "Miscellaneous");
            });

            ui.separator();

            let mh = 208.0;
            let r = ui.allocate_ui(Vec2::new(1000.0, mh), |ui| match gui_tab {
                GuiTab::Keyboard => {
                    ui.label("TODO!");
                }
                GuiTab::System => {
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
                }
                GuiTab::Miscellaneous => {
                    ui.label("TODO!");
                    ui.set_enabled(connection_status == ConnectionStatus::Connected);
                    if ui.button("Update JukeBox").clicked() {
                        serialcommand_tx1
                            .send(SerialCommand::UpdateDevice)
                            .expect("failed to send update command");
                    }
                    if ui.button("Test Function 0").clicked() {
                        serialcommand_tx1
                            .send(SerialCommand::TestFunction0)
                            .expect("failed to send test command");
                    }
                }
            });
            let h = r.response.rect.height();
            if h < mh {
                ui.allocate_space(Vec2::new(0.0, mh - h));
            }

            ui.separator();

            ui.label(format!("Friend Team Inc. (c) 2024"));

            ui.separator();
        });

        // Call a new frame every frame, bypassing the limited updates.
        // NOTE: This is a bad idea, we should probably change this later
        // and only update the window as necessary.
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
