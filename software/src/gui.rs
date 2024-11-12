// Graphical User Interface (pronounced like GIF)

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{
    vec2, Align, Button, CentralPanel, Color32, ComboBox, Grid, Layout, RichText, SelectableLabel,
    Sense, TextBuffer, TextEdit, Ui, ViewportBuilder,
};
use egui_phosphor::regular as phos;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::reaction::{InputKey, Peripheral, ReactionConfig, ReactionMetaTest};
use crate::serial::{serial_task, SerialCommand, SerialConnectionDetails, SerialEvent};
use crate::splash::SPLASH_MESSAGES;

#[derive(PartialEq)]
enum GuiTab {
    Device,
    Settings,
}

#[derive(PartialEq)]
enum GuiDeviceTab {
    None,
    Keyboard,
    Knobs1,
    Knobs2,
    Pedal1,
    Pedal2,
    Pedal3,
}

#[derive(PartialEq)]
enum ConnectionStatus {
    Connected,
    LostConnection,
    Disconnected,
}

#[derive(Serialize, Deserialize)]
struct JukeBoxConfig {
    current_profile: String,
    profiles: HashMap<String, HashMap<InputKey, ReactionConfig>>,
}

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct JukeBoxGui {
    splash_timer: Instant,
    splash_index: usize,

    conn_status: ConnectionStatus,

    gui_tab: GuiTab,
    device_tab: GuiDeviceTab,

    device_info: Option<SerialConnectionDetails>,
    device_peripherals: HashSet<Peripheral>,

    config: JukeBoxConfig,
    config_renaming_profile: bool,
    config_profile_name_entry: String,
}
impl JukeBoxGui {
    fn new() -> Self {
        // TODO: rework later for file configs
        let config: JukeBoxConfig = JukeBoxConfig {
            current_profile: "Profile 1".to_string(),
            profiles: HashMap::from([
                (
                    "Profile 1".to_string(),
                    HashMap::from([(
                        InputKey::KeyboardSwitch1,
                        ReactionConfig::MetaTest(ReactionMetaTest {}),
                    )]),
                ),
                (
                    "Profile 3".to_string(),
                    HashMap::from([(
                        InputKey::KeyboardSwitch12,
                        ReactionConfig::MetaTest(ReactionMetaTest {}),
                    )]),
                ),
            ]),
        };

        JukeBoxGui {
            splash_timer: Instant::now(),
            splash_index: 0usize,
            conn_status: ConnectionStatus::Disconnected,
            gui_tab: GuiTab::Device,
            device_tab: GuiDeviceTab::None,
            device_peripherals: HashSet::new(),
            device_info: None,
            config: config,
            config_renaming_profile: false,
            config_profile_name_entry: String::new(),
        }
    }

    fn run(mut self) {
        // channels cannot be a part of Self due to partial move errors
        let (s_evnt_tx, s_evnt_rx) = channel::<SerialEvent>(); // serialcomms thread sends events to gui thread
        let (s_cmd_tx, s_cmd_rx) = channel::<SerialCommand>(); // gui thread sends commands to serialcomms thread
        let (brkr_tx, brkr_rx) = channel::<bool>(); // ends serialcomms thread from gui
        let s_cmd_tx2 = s_cmd_tx.clone();

        // serial comms thread
        let serialcomms = thread::spawn(move || serial_task(&brkr_rx, &s_cmd_rx, &s_evnt_tx));

        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_title("JukeBox Desktop")
                .with_inner_size([960.0, 640.0])
                .with_maximize_button(false)
                .with_resizable(false)
                .with_icon(
                    eframe::icon_data::from_png_bytes(
                        &include_bytes!("../../assets/applogo.png")[..],
                    )
                    .unwrap(),
                ),
            centered: true,
            ..Default::default()
        };

        eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
            let mut fonts = eframe::egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            ctx.set_fonts(fonts);
            ctx.set_zoom_factor(2.0);

            while let Ok(event) = s_evnt_rx.try_recv() {
                match event {
                    SerialEvent::Connected(d) => {
                        self.conn_status = ConnectionStatus::Connected;
                        self.device_info = Some(d);
                    }
                    SerialEvent::LostConnection => {
                        self.conn_status = ConnectionStatus::LostConnection;
                        self.device_tab = GuiDeviceTab::None;
                        self.device_peripherals.clear();
                        self.device_info = None;
                    }
                    SerialEvent::Disconnected => {
                        self.conn_status = ConnectionStatus::Disconnected;
                        self.device_tab = GuiDeviceTab::None;
                        self.device_peripherals.clear();
                        self.device_info = None;
                    }
                    SerialEvent::GetPeripherals(p) => {
                        self.device_peripherals = p;
                        if self.device_peripherals.contains(&Peripheral::Keyboard) {
                            self.device_tab = GuiDeviceTab::Keyboard;
                        } else {
                            self.device_tab = GuiDeviceTab::None;
                        }
                    }
                    _ => todo!(),
                }
            }

            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.draw_profile_management(ui);
                    self.draw_settings_toggle(ui); // TODO: hide button when editing reaction
                });

                ui.separator();

                ui.allocate_ui(vec2(464.0, 252.0), |ui| match self.gui_tab {
                    GuiTab::Device => {
                        match self.device_tab {
                            GuiDeviceTab::Keyboard => {
                                self.draw_keyboard(ui);
                            }
                            GuiDeviceTab::Knobs1 | GuiDeviceTab::Knobs2 => {
                                ui.allocate_exact_size(vec2(324.0, 231.0), Sense::hover());
                            }
                            GuiDeviceTab::Pedal1 | GuiDeviceTab::Pedal2 | GuiDeviceTab::Pedal3 => {
                                ui.allocate_exact_size(vec2(324.0, 231.0), Sense::hover());
                            }
                            GuiDeviceTab::None => {
                                ui.allocate_exact_size(vec2(324.0, 231.0), Sense::hover());
                            }
                        }

                        self.draw_peripheral_tabs(ui, &s_cmd_tx);
                    }
                    GuiTab::Settings => {
                        self.draw_jukebox_logo(ui);
                        ui.label("");
                        ui.label("");
                        self.draw_update_button(ui, &s_cmd_tx);
                        ui.label("");
                        self.draw_testfunc_button(ui, &s_cmd_tx);
                        self.draw_settings_bottom(ui);
                    }
                });

                ui.separator();

                self.draw_splash_text(ui);
            });

            // Call a new frame every frame, bypassing the limited updates.
            // NOTE: This is a bad idea, we should probably change this later
            // and only update the window as necessary.
            ctx.request_repaint();
        })
        .expect("eframe error");

        brkr_tx.send(true).expect("could not send breaker 2 signal");

        s_cmd_tx2
            .send(SerialCommand::DisconnectDevice)
            .expect("could not send disconnect signal");

        serialcomms
            .join()
            .expect("could not rejoin serialcomms thread");
    }

    fn draw_profile_management(&mut self, ui: &mut Ui) {
        ui.scope(|ui| {
            // TODO
            // if editing_key_reaction {
            //     ui.disable();
            // }
            if self.gui_tab != GuiTab::Device {
                ui.disable();
            }

            // Profile select/edit
            if self.config_renaming_profile {
                // TODO: this shifts everything down a bit too much, fix later
                let edit = ui.add(
                    TextEdit::singleline(&mut self.config_profile_name_entry).desired_width(142.0),
                );
                if edit.lost_focus() && self.config_profile_name_entry.len() > 0 {
                    self.config_renaming_profile = false;
                    let c = self
                        .config
                        .profiles
                        .remove(&self.config.current_profile)
                        .expect("");
                    self.config
                        .profiles
                        .insert(self.config_profile_name_entry.to_string(), c);
                    self.config
                        .current_profile
                        .replace_range(.., &self.config_profile_name_entry);
                }
                if !edit.has_focus() {
                    edit.request_focus();
                }
            } else {
                ComboBox::from_id_salt("ProfileSelect")
                    .selected_text(self.config.current_profile.clone()) // TODO: show current profile name here
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for (k, _) in &self.config.profiles {
                            let u = ui.add(SelectableLabel::new(
                                *k == self.config.current_profile.clone(),
                                &*k.clone(),
                            ));
                            if u.clicked() {
                                self.config.current_profile = k.to_string();
                            }
                        }
                    })
                    .response
                    .on_hover_text_at_pointer("Profie Select");
            }

            // Profile management
            ui.scope(|ui| {
                if self.config_renaming_profile {
                    ui.disable();
                }

                let new_btn = ui
                    .button(RichText::new(phos::PLUS_CIRCLE))
                    .on_hover_text_at_pointer("New Profile");
                if new_btn.clicked() {
                    let mut idx = self.config.profiles.keys().len() + 1;
                    loop {
                        let name = format!("Profile {}", idx);
                        if !self.config.profiles.contains_key(&name) {
                            self.config.profiles.insert(name, HashMap::new());
                            // TODO: immediately save config to file
                            break;
                        }
                        idx += 1;
                    }
                }
            });

            ui.scope(|ui| {
                if self.config_renaming_profile {
                    ui.disable();
                }

                let edit_btn = ui
                    .button(RichText::new(phos::NOTE_PENCIL))
                    .on_hover_text_at_pointer("Edit Profile Name");
                if edit_btn.clicked() {
                    self.config_renaming_profile = true;
                    self.config_profile_name_entry
                        .replace_with(&self.config.current_profile);
                }
            });

            ui.scope(|ui| {
                if self.config_renaming_profile {
                    ui.disable();
                }

                if self.config.profiles.keys().len() <= 1 {
                    ui.disable();
                }
                let delete_btn = ui
                    .button(RichText::new(phos::TRASH))
                    .on_hover_text_at_pointer("Delete Profile");
                if delete_btn.clicked() {
                    // TODO: check other profiles and make sure they dont rely on this profile
                    self.config.profiles.remove(&self.config.current_profile);
                    self.config.current_profile =
                        self.config.profiles.keys().next().expect("").to_string();
                    // TODO: immediately save config to file
                }
            });
        });
    }

    fn draw_settings_toggle(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let settings_btn = ui
                .selectable_label(
                    self.gui_tab == GuiTab::Settings,
                    RichText::new(phos::GEAR_FINE),
                )
                .on_hover_text_at_pointer("Settings");
            if settings_btn.clicked() {
                match self.gui_tab {
                    GuiTab::Device => self.gui_tab = GuiTab::Settings,
                    GuiTab::Settings => self.gui_tab = GuiTab::Device,
                }
            }
        });
    }

    fn draw_keyboard(&mut self, ui: &mut Ui) {
        let s = Sense::hover();
        ui.horizontal(|ui| {
            ui.allocate_exact_size([62.0, 0.0].into(), s);
            Grid::new("KBGrid").show(ui, |ui| {
                let col = 4;
                let row = 3;
                for y in 0..row {
                    for x in 0..col {
                        let btn = Button::new(format!("F{}", 12 + x + y * col + 1));
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

    fn draw_peripheral_tabs(&mut self, ui: &mut Ui, s_cmd_tx: &Sender<SerialCommand>) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                if ui.button(RichText::new(phos::ARROW_CLOCKWISE)).clicked() {
                    s_cmd_tx
                        .send(SerialCommand::GetPeripherals)
                        .expect("failed to send get peripherals command");
                }

                let mut f = |t1, t2, s: &str| {
                    if self.device_peripherals.contains(t1) {
                        ui.selectable_value(&mut self.device_tab, t2, s);
                    }
                };
                f(&Peripheral::Pedal3, GuiDeviceTab::Pedal3, "Pedal 3");
                f(&Peripheral::Pedal2, GuiDeviceTab::Pedal2, "Pedal 2");
                f(&Peripheral::Pedal1, GuiDeviceTab::Pedal1, "Pedal 1");
                f(&Peripheral::Knobs2, GuiDeviceTab::Knobs2, "Knobs 2");
                f(&Peripheral::Knobs1, GuiDeviceTab::Knobs1, "Knobs 1");
                f(&Peripheral::Keyboard, GuiDeviceTab::Keyboard, "Keyboard");
            });
        });
    }

    fn draw_jukebox_logo(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("JukeBox Desktop")
                    .heading()
                    .color(Color32::from_rgb(255, 200, 100)),
            );
            ui.label(format!("-  v{}", APP_VERSION));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let res = match self.conn_status {
                    ConnectionStatus::Connected => ("Connected.", Color32::from_rgb(50, 200, 50)),
                    ConnectionStatus::Disconnected => {
                        ("Not connected.", Color32::from_rgb(200, 200, 50))
                    }
                    ConnectionStatus::LostConnection => {
                        ("Lost connection!", Color32::from_rgb(200, 50, 50))
                    }
                };

                ui.label(RichText::new(res.0).color(res.1));
            });
        });
    }

    fn draw_update_button(&mut self, ui: &mut Ui, s_cmd_tx: &Sender<SerialCommand>) {
        ui.horizontal(|ui| {
            if self.conn_status != ConnectionStatus::Connected {
                ui.disable();
            }
            if ui.button("Update JukeBox").clicked() {
                s_cmd_tx
                    .send(SerialCommand::UpdateDevice)
                    .expect("failed to send update command");
            }
            ui.label(" - ");
            ui.label("Reboots the connected JukeBox into Update Mode.")
        });
    }

    fn draw_testfunc_button(&mut self, ui: &mut Ui, s_cmd_tx: &Sender<SerialCommand>) {
        ui.horizontal(|ui| {
            if self.conn_status != ConnectionStatus::Connected {
                ui.disable();
            }
            if ui.button("Debug Signal").clicked() {
                s_cmd_tx
                    .send(SerialCommand::TestFunction)
                    .expect("failed to send test command");
            }
            ui.label(" - ");
            ui.label("Send debug signal to JukeBox.")
        });
    }

    fn draw_settings_bottom(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                if let Some(i) = &self.device_info {
                    ui.label(format!("Firmware Version: {}", i.firmware_version));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label("Made w/ <3 by Friend Team Inc. (c) 2024");
                });
            });

            ui.horizontal(|ui| {
                if let Some(i) = &self.device_info {
                    ui.label(format!("Device UID: {}", i.device_uid));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.hyperlink_to("Donate", "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
                    ui.label(" - ");
                    ui.hyperlink_to("Repository", "https://github.com/FriendTeamInc/JukeBox");
                    ui.label(" - ");
                    ui.hyperlink_to("Homepage", "https://friendteam.biz");
                });
            });
        });
    }

    fn draw_splash_text(&mut self, ui: &mut Ui) {
        if Instant::now() > self.splash_timer {
            loop {
                let new_index = rand::thread_rng().gen_range(0..SPLASH_MESSAGES.len());
                if new_index != self.splash_index {
                    self.splash_index = new_index;
                    break;
                }
            }
            self.splash_timer = Instant::now() + Duration::from_secs(30);
        }
        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
            ui.label(
                RichText::new(SPLASH_MESSAGES[self.splash_index])
                    .monospace()
                    .size(6.0),
            );
        });
    }
}

pub fn basic_gui() {
    JukeBoxGui::new().run();
}
