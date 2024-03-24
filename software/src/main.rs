// An application for interfacing with a JukeBox over serial.

// #![windows_subsystem = "windows"] // disables console spawning for release build

mod gui;
mod serial;
mod splash;
mod system;
mod util;

use crate::util::{ExitCode, ExitMsg};

fn deffered_main() -> Result<(), ExitMsg> {
    // Setup the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(0)
        .init()
        .map_err(|e| {
            ExitMsg::new(
                ExitCode::CannotInitStderrLogger,
                format!(
                    "Failed to initialize stderr logger, reason: \"{}\".",
                    e.to_string()
                ),
            )
        })?;

    gui::basic_gui();

    Ok(())
}

fn main() {
    std::process::exit(deffered_main().map_or_else(
        |err| {
            log::error!("{}", err);
            err.code as i32
        },
        |_| 0,
    ))
}
