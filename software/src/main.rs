// A lightweight program to simulate JukeBox serial communication.

mod serial;
mod util;

use serialport::SerialPortType;

use crate::util::{ExitCode, ExitMsg};

fn deffered_main() -> Result<(), ExitMsg> {
    // Setup the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(5)
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

    let ports = serialport::available_ports().map_err(|why| {
        ExitMsg::new(
            ExitCode::GenericError,
            format!("Failed to enumerate serial ports, reason: \"{}\".", why),
        )
    })?;
    let ports: Vec<_> = ports
        .iter()
        .filter(|p| match &p.port_type {
            SerialPortType::UsbPort(p) => p.pid == 0xF20A && p.vid == 0x1209,
            _ => false,
        })
        .collect();
    log::info!(
        "Found ports: {:?}",
        ports
            .iter()
            .map(|f| f.port_name.clone())
            .collect::<Vec<_>>()
    );
    if ports.len() == 0 {
        return Err(ExitMsg::new(
            ExitCode::GenericError,
            format!("Failed to find JukeBox serial port."),
        ));
    }
    let port = ports.get(0).unwrap(); // TODO: provide an argument to choose from this vector

    let mut f = serialport::new(port.port_name.clone(), 115200)
        // .timeout(Duration::from_millis(10))
        .open()
        .map_err(|why| {
            ExitMsg::new(
                ExitCode::GenericError,
                format!("Failed to open serial port, reason: \"{}\".", why),
            )
        })?;

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
