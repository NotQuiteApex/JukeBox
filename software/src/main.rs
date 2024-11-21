// A desktop application for interfacing with a JukeBox over serial.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // disables console spawning for release build

mod gui;
mod reaction;
mod serial;
mod splash;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    gui::basic_gui();

    Ok(())
}
