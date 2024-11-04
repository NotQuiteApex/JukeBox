use std::ops::Add;
use std::thread;
use std::time::Instant;
use std::{sync::mpsc::channel, time::Duration};

use eframe::egui::{
    vec2, Align, CentralPanel, Color32, Layout, RichText, ViewportBuilder, ViewportCommand,
};

use rand::prelude::*;

use crate::serial::{serial_get_device, serial_task, SerialCommand, SerialEvent};
use crate::splash::SPLASH_MESSAGES;

#[derive(PartialEq)]
enum ConnectionStatus {
    Connected,
    NotConnected,
    LostConnection,
}

#[derive(PartialEq)]
enum GuiTab {
    Device,
    Settings,
}

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn basic_gui() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("JukeBox Desktop")
            .with_inner_size([960.0, 640.0])
            .with_maximize_button(false)
            .with_resizable(false)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/applogo.png")[..])
                    .unwrap(),
            ),
        centered: true,
        ..Default::default()
    };

    let (serialevent_tx, serialevent_rx) = channel::<SerialEvent>(); // serialcomms thread sends events to gui thread
    let (serialcommand_tx, serialcommand_rx) = channel::<SerialCommand>(); // gui thread sends commands to serialcomms thread

    let (breaker_tx, breaker_rx) = channel::<bool>(); // ends serialcomms thread from gui

    let mut splash_message_timer = Instant::now();
    let mut splash_message_index = 0usize;

    // serial comms thread
    let serialcomms = thread::spawn(move || {
        // TODO: check application cpu usage when device is connected
        loop {
            if let Ok(_) = breaker_rx.try_recv() {
                break;
            }

            let f = serial_get_device();
            if let Err(_) = f {
                // log::error!("Failed to get serial device. Error: `{}`.", e);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            let mut f = f.unwrap();

            match serial_task(&mut f, &serialcommand_rx, &serialevent_tx) {
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
    let serialcommand_tx1 = serialcommand_tx.clone();

    let mut gui_tab = GuiTab::Device;

    eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
        ctx.set_zoom_factor(2.0); // TODO: find a better solution, the first frame is always the wrong size

        while let Ok(event) = serialevent_rx.try_recv() {
            match event {
                SerialEvent::Connected => connection_status = ConnectionStatus::Connected,
                SerialEvent::LostConnection => connection_status = ConnectionStatus::LostConnection,
                SerialEvent::Disconnected => connection_status = ConnectionStatus::NotConnected,
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut gui_tab, GuiTab::Device, "Device");
                ui.selectable_value(&mut gui_tab, GuiTab::Settings, "Settings");
            });

            ui.separator();

            let mw = 464.0;
            let mh = 246.0;
            ui.allocate_ui(vec2(mw, mh), |ui| match gui_tab {
                GuiTab::Device => {
                    let col = 4;
                    let row = 3;
                    let h = mh / 3.0 - 2.0;
                    for y in 0..row {
                        ui.columns(col, |c| {
                            for x in 0..col {
                                c[x].set_min_height(h);
                                c[x].set_max_height(h);
                                c[x].centered_and_justified(|ui| {
                                    if ui.button(format!("F{}", 12 + x + y * col + 1)).clicked() {
                                        log::info!("({}, {}) clicked", x + 1, y + 1);
                                        // TODO: add config menu when button is clicked
                                        // TODO: highlight button when press signal is recieved
                                        // TODO: display some better text in the buttons
                                    }
                                });
                            }
                        });
                    }
                }
                GuiTab::Settings => {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("JukeBox Desktop")
                                .heading()
                                .color(Color32::from_rgb(255, 200, 100)),
                        );
                        ui.label(format!(" - v{}", APP_VERSION));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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

                            ctx.send_viewport_cmd(ViewportCommand::Title(format!(
                                "JukeBox Desktop - v{} - {}",
                                APP_VERSION, res.0
                            )));
                        });
                    });

                    ui.separator();
                    ui.label("");

                    ui.horizontal(|ui| {
                        if connection_status != ConnectionStatus::Connected {
                            ui.disable();
                        }
                        if ui.button("Update JukeBox").clicked() {
                            serialcommand_tx1
                                .send(SerialCommand::UpdateDevice)
                                .expect("failed to send update command");
                        }
                        ui.label(" - ");
                        ui.label("Reboots the connected JukeBox into Update Mode.")
                    });

                    ui.label("");

                    ui.horizontal(|ui| {
                        if connection_status != ConnectionStatus::Connected {
                            ui.disable();
                        }
                        if ui.button("Debug Signal").clicked() {
                            serialcommand_tx1
                                .send(SerialCommand::TestFunction0)
                                .expect("failed to send test command");
                        }
                        ui.label(" - ");
                        ui.label("Send debug signal to JukeBox.")
                    });

                    ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                        ui.label("Made w/ <3 by Friend Team Inc. (c) 2024");
                        ui.horizontal(|ui| {
                            ui.hyperlink_to(
                                "Donate",
                                "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                            );
                            ui.label(" - ");
                            ui.hyperlink_to(
                                "Repository",
                                "https://github.com/FriendTeamInc/JukeBox",
                            );
                            ui.label(" - ");
                            ui.hyperlink_to("Homepage", "https://friendteam.biz");
                        })
                    });
                }
            });

            ui.separator();

            if Instant::now() > splash_message_timer {
                loop {
                    let new_index = rand::thread_rng().gen_range(0..SPLASH_MESSAGES.len());
                    if new_index != splash_message_index {
                        splash_message_index = new_index;
                        break;
                    }
                }
                splash_message_timer = Instant::now().add(Duration::from_secs(10));
            }
            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                ui.monospace(SPLASH_MESSAGES[splash_message_index]);
            });

            ui.separator();
        });

        // Call a new frame every frame, bypassing the limited updates.
        // NOTE: This is a bad idea, we should probably change this later
        // and only update the window as necessary.
        ctx.request_repaint();
    })
    .expect("eframe error");

    breaker_tx
        .send(true)
        .expect("could not send breaker 2 signal");

    serialcommand_tx
        .send(SerialCommand::DisconnectDevice)
        .expect("could not send disconnect signal");

    serialcomms
        .join()
        .expect("could not rejoin serialcomms thread");
}
