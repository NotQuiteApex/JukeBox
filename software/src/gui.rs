use std::collections::HashMap;
use std::thread;
use std::time::Instant;
use std::{sync::mpsc::channel, time::Duration};

use eframe::egui::{
    vec2, Align, Button, CentralPanel, Color32, ComboBox, Grid, Layout, RichText, SelectableLabel,
    Sense, ViewportBuilder, ViewportCommand,
};
use egui_phosphor::regular as phos;

use rand::prelude::*;

use crate::reaction::{InputKey, ReactionConfig};
use crate::serial::{
    serial_get_device, serial_task, JukeBoxPeripherals, SerialCommand, SerialEvent,
};
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

#[derive(PartialEq)]
enum GuiDeviceTab {
    Keyboard,
    Knobs1,
    Knobs2,
    Pedal1,
    Pedal2,
    Pedal3,
}

// TODO: manage this with serde json
struct _JukeBoxConfig {
    profiles: HashMap<String, HashMap<InputKey, ReactionConfig>>,
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

    // TODO: make mut later when firmware supports reporting it
    let connected_peripherals = vec![
        JukeBoxPeripherals::Keyboard,
        JukeBoxPeripherals::Knobs1,
        JukeBoxPeripherals::Knobs2,
        JukeBoxPeripherals::Pedal1,
        JukeBoxPeripherals::Pedal2,
        JukeBoxPeripherals::Pedal3,
    ];

    let mut gui_tab = GuiTab::Device;
    let mut gui_device_tab = GuiDeviceTab::Keyboard;

    eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
        let mut fonts = eframe::egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
        ctx.set_zoom_factor(2.0);

        while let Ok(event) = serialevent_rx.try_recv() {
            match event {
                SerialEvent::Connected => connection_status = ConnectionStatus::Connected,
                SerialEvent::LostConnection => connection_status = ConnectionStatus::LostConnection,
                SerialEvent::Disconnected => connection_status = ConnectionStatus::NotConnected,
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Profile select
                // TODO: disable select when editing reaction
                ComboBox::from_label("ProfileSelect")
                    .selected_text("Profile Select") // TODO: show current profile name here
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        // TODO: populate dynamically
                        ui.add(SelectableLabel::new(true, "Profile 1"));
                        ui.add(SelectableLabel::new(false, "Profile 2"));
                        ui.add(SelectableLabel::new(false, "Profile 4"));
                    })
                    .response
                    .on_hover_text_at_pointer("Profie Select");

                // Profile management
                // TODO: hide buttons when editing reaction
                let new_btn = ui
                    .button(RichText::new(phos::PLUS_CIRCLE))
                    .on_hover_text_at_pointer("New Profile");
                let edit_btn = ui
                    .button(RichText::new(phos::NOTE_PENCIL))
                    .on_hover_text_at_pointer("Edit Profile Name");
                let save_btn = ui
                    .button(RichText::new(phos::FLOPPY_DISK))
                    .on_hover_text_at_pointer("Save Profile");
                let delete_btn = ui
                    .button(RichText::new(phos::TRASH))
                    .on_hover_text_at_pointer("Delete Profile");
                if new_btn.clicked() {
                    log::info!("New Profile button clicked!");
                }
                if edit_btn.clicked() {
                    log::info!("Edit Profile Name button clicked!");
                }
                if save_btn.clicked() {
                    log::info!("Save Profile button clicked!");
                }
                if delete_btn.clicked() {
                    log::info!("Delete Profile button clicked!");
                }

                // Settings page toggle
                // TODO: hide button when editing reaction
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let settings_btn = ui
                        .selectable_label(
                            gui_tab == GuiTab::Settings,
                            RichText::new(phos::GEAR_FINE),
                        )
                        .on_hover_text_at_pointer("Settings");
                    if settings_btn.clicked() {
                        match gui_tab {
                            GuiTab::Device => gui_tab = GuiTab::Settings,
                            GuiTab::Settings => gui_tab = GuiTab::Device,
                        }
                    }
                });
            });

            ui.separator();

            let mw = 464.0;
            let mh = 252.0;
            ui.allocate_ui(vec2(mw, mh), |ui| match gui_tab {
                GuiTab::Device => {
                    match gui_device_tab {
                        GuiDeviceTab::Keyboard => {
                            let s = Sense::hover();
                            ui.horizontal(|ui| {
                                ui.allocate_exact_size([(mw - 340.0) / 2.0, 0.0].into(), s);
                                Grid::new("KBGrid").show(ui, |ui| {
                                    let col = 4;
                                    let row = 3;
                                    for y in 0..row {
                                        for x in 0..col {
                                            let btn =
                                                Button::new(format!("F{}", 12 + x + y * col + 1));
                                            let btn = ui.add_sized([75.0, 75.0], btn);
                                            if btn.clicked() {
                                                log::info!("({}, {}) clicked", x + 1, y + 1);
                                                // TODO: add config menu when button is clicked
                                                // TODO: highlight button when press signal is recieved
                                                // TODO: display some better text in the buttons
                                                // TODO: add hover text for button info
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                            });
                        }
                        GuiDeviceTab::Knobs1 | GuiDeviceTab::Knobs2 => {
                            ui.allocate_exact_size(vec2(324.0, 231.0), Sense::hover());
                        }
                        GuiDeviceTab::Pedal1 | GuiDeviceTab::Pedal2 | GuiDeviceTab::Pedal3 => {
                            ui.allocate_exact_size(vec2(324.0, 231.0), Sense::hover());
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            if ui.button(RichText::new(phos::ARROW_CLOCKWISE)).clicked() {
                                serialcommand_tx1
                                    .send(SerialCommand::RefreshPeripherals)
                                    .expect("failed to send refresh peripherals command");
                            }

                            for p in connected_peripherals.iter() {
                                let p = match p {
                                    JukeBoxPeripherals::Keyboard => {
                                        (GuiDeviceTab::Pedal3, "Pedal 3")
                                    }
                                    JukeBoxPeripherals::Knobs1 => (GuiDeviceTab::Pedal2, "Pedal 2"),
                                    JukeBoxPeripherals::Knobs2 => (GuiDeviceTab::Pedal1, "Pedal 1"),
                                    JukeBoxPeripherals::Pedal1 => (GuiDeviceTab::Knobs2, "Knobs 2"),
                                    JukeBoxPeripherals::Pedal2 => (GuiDeviceTab::Knobs1, "Knobs 1"),
                                    JukeBoxPeripherals::Pedal3 => {
                                        (GuiDeviceTab::Keyboard, "Keyboard")
                                    }
                                };
                                ui.selectable_value(&mut gui_device_tab, p.0, p.1);
                            }
                        });
                    });
                }
                GuiTab::Settings => {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("JukeBox Desktop")
                                .heading()
                                .color(Color32::from_rgb(255, 200, 100)),
                        );
                        ui.label(format!("-  v{}", APP_VERSION));
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

                    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Firmware Version: TODO");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label("Made w/ <3 by Friend Team Inc. (c) 2024");
                            });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Serial ID: TODO");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                            });
                        });
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
                splash_message_timer = Instant::now() + Duration::from_secs(30);
            }
            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                ui.label(
                    RichText::new(SPLASH_MESSAGES[splash_message_index])
                        .monospace()
                        .size(6.0),
                );
            });
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
