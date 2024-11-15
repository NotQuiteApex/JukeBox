// Graphical User Interface (pronounced like GIF)

use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{
    vec2, Align, Button, CentralPanel, Color32, ComboBox, Grid, Layout, RichText, Rounding, Sense,
    TextBuffer, TextEdit, Ui, ViewportBuilder,
};
use egui_phosphor::regular as phos;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::reaction::{reaction_task, InputKey, Peripheral, ReactionConfig};
use crate::serial::{serial_task, SerialCommand, SerialConnectionDetails, SerialEvent};
use crate::splash::SPLASH_MESSAGES;

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

#[derive(Serialize, Deserialize, Clone)]
pub struct JukeBoxConfig {
    pub current_profile: String,
    pub profiles: HashMap<String, HashMap<InputKey, ReactionConfig>>,
}
impl Default for JukeBoxConfig {
    fn default() -> Self {
        JukeBoxConfig {
            current_profile: "Default".to_string(),
            profiles: HashMap::from([("Default".to_string(), HashMap::new())]),
        }
    }
}
impl JukeBoxConfig {
    fn get_path() -> PathBuf {
        let mut p = dirs::config_dir().expect("failed to find config directory");
        p.push("JukeBoxDesktop");
        create_dir_all(&p).expect("failed to create config directory");
        p.push("config.json");
        p
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        let file = match File::open(path) {
            Err(_) => {
                return JukeBoxConfig::default();
            }
            Ok(f) => f,
        };

        let conf = match serde_json::from_reader(file) {
            Err(_) => {
                return JukeBoxConfig::default();
            }
            Ok(c) => c,
        };

        // TODO: serde_validate the config?

        conf
    }

    pub fn save(&self) {
        let path = Self::get_path();
        let file = File::create(path).expect("failed to create config file");
        serde_json::to_writer(file, &self).expect("failed to write config file");
    }
}

struct JukeBoxGui {
    splash_timer: Instant,
    splash_index: usize,

    conn_status: ConnectionStatus,

    gui_tab: GuiTab,
    device_tab: GuiDeviceTab,

    device_info: Option<SerialConnectionDetails>,
    device_peripherals: HashSet<Peripheral>,
    device_inputs: HashSet<InputKey>,

    config: Arc<Mutex<JukeBoxConfig>>,
    config_renaming_profile: bool,
    config_profile_name_entry: String,
}
impl JukeBoxGui {
    fn new() -> Self {
        // TODO: rework later for file configs
        let config = JukeBoxConfig::load();
        config.save();
        let config = Arc::new(Mutex::new(JukeBoxConfig::load()));

        JukeBoxGui {
            splash_timer: Instant::now(),
            splash_index: 0usize,
            conn_status: ConnectionStatus::Disconnected,
            gui_tab: GuiTab::Device,
            device_tab: GuiDeviceTab::None,
            device_peripherals: HashSet::new(),
            device_inputs: HashSet::new(),
            device_info: None,
            config: config,
            config_renaming_profile: false,
            config_profile_name_entry: String::new(),
        }
    }

    fn run(mut self) {
        // channels cannot be a part of Self due to partial move errors
        let (s_evnt_tx, s_evnt_rx) = channel::<SerialEvent>(); // serial thread sends events to reaction thread
        let (r_evnt_tx, r_evnt_rx) = channel::<SerialEvent>(); // reaction thread sends events to gui thread
        let (s_cmd_tx, s_cmd_rx) = channel::<SerialCommand>(); // gui thread sends commands to serial thread

        let brkr = Arc::new(AtomicBool::new(false)); // ends other threads from gui
        let brkr_serial = brkr.clone();
        let brkr_reaction = brkr.clone();

        let s_evnt_tx_serial = s_evnt_tx.clone();
        let s_cmd_tx2 = s_cmd_tx.clone();

        let config_reaction = self.config.clone();

        // serial comms thread
        let serialcomms =
            thread::spawn(move || serial_task(brkr_serial, s_cmd_rx, s_evnt_tx_serial));

        // reaction comms thread
        let reactioncomms = thread::spawn(move || {
            reaction_task(brkr_reaction, s_evnt_rx, r_evnt_tx, config_reaction)
        });

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
            ctx.set_zoom_factor(2.0);
            let mut fonts = eframe::egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            ctx.set_fonts(fonts);

            self.handle_serial_events(&r_evnt_rx);

            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.draw_profile_management(ui);
                    self.draw_settings_toggle(ui); // TODO: hide button when editing reaction
                });

                ui.separator();

                ui.allocate_ui(vec2(464.0, 252.0), |ui| match self.gui_tab {
                    GuiTab::Device => self.draw_device_page(ui, &s_cmd_tx),
                    GuiTab::Settings => self.draw_settings_page(ui, &s_cmd_tx),
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

        brkr.store(true, std::sync::atomic::Ordering::Relaxed);

        s_cmd_tx2
            .send(SerialCommand::DisconnectDevice)
            .expect("could not send disconnect signal");

        let _ = serialcomms
            .join()
            .expect("could not rejoin serialcomms thread");

        let _ = reactioncomms
            .join()
            .expect("could not rejoin reactioncomms thread");
    }

    fn handle_serial_events(&mut self, s_evnt_rx: &Receiver<SerialEvent>) {
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
                SerialEvent::GetInputKeys(k) => {
                    self.device_inputs = k
                    // TODO: run all config.profiles[config.current_profile] actions
                } // _ => todo!(),
            }
        }
    }

    fn draw_device_page(&mut self, ui: &mut Ui, s_cmd_tx: &Sender<SerialCommand>) {
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

    fn draw_settings_page(&mut self, ui: &mut Ui, s_cmd_tx: &Sender<SerialCommand>) {
        self.draw_jukebox_logo(ui);
        ui.label("");
        ui.label("");
        self.draw_update_button(ui, &s_cmd_tx);
        ui.label("");
        self.draw_testfunc_button(ui, &s_cmd_tx);
        self.draw_settings_bottom(ui);
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
                    let mut conf = self.config.lock().unwrap();

                    let p = conf.current_profile.clone();
                    let c = conf.profiles.remove(&p).expect("");
                    conf.profiles
                        .insert(self.config_profile_name_entry.to_string(), c);
                    conf.current_profile
                        .replace_range(.., &self.config_profile_name_entry);
                }
                if !edit.has_focus() {
                    edit.request_focus();
                }
            } else {
                let mut conf = self.config.lock().unwrap();
                let profiles = conf.profiles.clone();
                let current = conf.current_profile.clone();
                ComboBox::from_id_salt("ProfileSelect")
                    .selected_text(conf.current_profile.clone()) // TODO: show current profile name here
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for (k, _) in &profiles {
                            let u = ui.selectable_label(*k == current, &*k.clone());
                            if u.clicked() {
                                conf.current_profile = k.to_string();
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
                    let mut conf = self.config.lock().unwrap();
                    let mut idx = conf.profiles.keys().len() + 1;
                    loop {
                        let name = format!("Profile {}", idx);
                        if !conf.profiles.contains_key(&name) {
                            conf.profiles.insert(name, HashMap::new());
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
                    let conf = self.config.lock().unwrap();
                    self.config_renaming_profile = true;
                    self.config_profile_name_entry
                        .replace_with(&conf.current_profile);
                }
            });

            ui.scope(|ui| {
                let mut conf = self.config.lock().unwrap();

                if self.config_renaming_profile {
                    ui.disable();
                }

                if conf.profiles.keys().len() <= 1 {
                    ui.disable();
                }
                let delete_btn = ui
                    .button(RichText::new(phos::TRASH))
                    .on_hover_text_at_pointer("Delete Profile");
                if delete_btn.clicked() {
                    // TODO: check other profiles and make sure they dont rely on this profile
                    let p = conf.current_profile.clone();
                    conf.profiles.remove(&p);
                    conf.current_profile = conf.profiles.keys().next().expect("").to_string();
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
                let keys = [
                    [
                        InputKey::KeyboardSwitch1,
                        InputKey::KeyboardSwitch2,
                        InputKey::KeyboardSwitch3,
                        InputKey::KeyboardSwitch4,
                    ],
                    [
                        InputKey::KeyboardSwitch5,
                        InputKey::KeyboardSwitch6,
                        InputKey::KeyboardSwitch7,
                        InputKey::KeyboardSwitch8,
                    ],
                    [
                        InputKey::KeyboardSwitch9,
                        InputKey::KeyboardSwitch10,
                        InputKey::KeyboardSwitch11,
                        InputKey::KeyboardSwitch12,
                    ],
                    // [
                    //     InputKey::KeyboardSwitch13,
                    //     InputKey::KeyboardSwitch14,
                    //     InputKey::KeyboardSwitch15,
                    //     InputKey::KeyboardSwitch16,
                    // ],
                ];
                for (y, k) in keys.iter().enumerate() {
                    for (x, k) in k.iter().enumerate() {
                        let s = format!("F{}", 12 + x + y * 4 + 1);
                        let rt = RichText::new(s).heading();
                        let mut b = Button::new(rt);
                        if self.device_inputs.contains(k) {
                            let r = 20.0;
                            b = b.rounding(Rounding {
                                nw: r,
                                ne: r,
                                sw: r,
                                se: r,
                            });
                        }
                        let btn = ui.add_sized([75.0, 75.0], b);

                        if btn.clicked() {
                            log::info!("F{} clicked", 12 + x + y * 4 + 1);
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
                let b = ui
                    .button(RichText::new(phos::ARROW_CLOCKWISE))
                    .on_hover_text_at_pointer("Refresh Peripherals");
                if b.clicked() {
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
