// A lightweight program to simulate JukeBox serial communication.

mod serial;
mod util;

use crate::util::{ExitCode, ExitMsg};

fn deffered_main() -> Result<(), ExitMsg> {
    // Setup the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        // .verbosity(args.verbose as usize)
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

    // TODO: replace this with clap
    let args: Vec<_> = std::env::args().collect();
    // let exe = args.get(0).map_or("jukeboxcli", |v| v.as_str());
    if args.len() != 2 {
        return Err(ExitMsg::new(
            ExitCode::GenericError,
            "Invalid arguments.".to_owned(),
        ));
    }

    let md = std::fs::metadata(&args[1]);
    if md.is_err() {
        return Err(ExitMsg::new(
            ExitCode::GenericError,
            "Failed to read FILE metadata.".to_owned(),
        ));
    }
    let md = md.unwrap();
    if md.is_dir() {
        return Err(ExitMsg::new(
            ExitCode::GenericError,
            "FILE cannot be a directory.".to_owned(),
        ));
    }

    let f = serialport::new(&args[1], 115200)
        // .timeout(Duration::from_millis(10))
        .open();
    if f.is_err() {
        return Err(ExitMsg::new(
            ExitCode::GenericError,
            "Cannot open FILE.".to_owned(),
        ));
    }
    let mut f = f.unwrap();

    serial::serial_task(&mut f)
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
