// A lightweight program to simulate JukeBox serial communication.

mod cli;
mod gui;
mod serial;
mod system;
mod util;

use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    system::PCSystem,
    util::{ExitCode, ExitMsg},
};

fn deffered_main() -> Result<(), ExitMsg> {
    // Parse arguments
    let cli = Cli::parse();

    // Setup the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(cli.verbose as usize)
        .init()
        .map_err(|e| {
            ExitMsg::new(
                ExitCode::StderrLogger,
                format!(
                    "Failed to initialize stderr logger, reason: \"{}\".",
                    e.to_string()
                ),
            )
        })?;

    // Setup the SIGINT handler
    ctrlc::set_handler(move || {
        let e = ExitMsg::new(ExitCode::Interrupted, " Interrupted!".to_owned());
        log::error!("{}", e);
        println!("{}", e);
        std::process::exit(e.code as i32);
    })
    .map_err(|why| {
        ExitMsg::new(
            ExitCode::CannotRegisterSignalHandler,
            format!(
                "Cannot register signal interrupt handler, reason: \"{}\".",
                why
            ),
        )
    })?;

    match cli.command {
        Commands::Probe => {
            PCSystem::new()?.get_report().log_report();
            Ok(())
        }
        Commands::Commune => {
            let mut f = serial::serial_get_device()?;
            serial::serial_task(&mut f)
        }
        Commands::Gui => {
            gui::basic_gui();
            Ok(())
        }
    }
}

fn main() {
    std::process::exit(deffered_main().map_or_else(
        |err| {
            log::error!("{}", err);
            println!("{}", err);
            err.code as i32
        },
        |_| 0,
    ))
}
