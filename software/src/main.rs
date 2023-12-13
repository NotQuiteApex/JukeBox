// A lightweight program to simulate JukeBox serial communication.

mod cli;
mod serial;
mod system;
mod util;

use clap::Parser;
use serialport::SerialPortType;
use sysinfo::{System, SystemExt};

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
            println!("Probing...");
            println!("");
            let mut pcs = PCSystem::new()?;
            // std::thread::sleep(<sysinfo::System as SystemExt>::MINIMUM_CPU_UPDATE_INTERVAL);
            // pcs.update();

            println!("CPU Name: ---- '{}'", pcs.cpu_name());
            println!("GPU Name: ---- '{}'", pcs.gpu_name());
            println!("Total Memory: - {} GiB", pcs.memory_total());
            println!("");
            println!("CPU Freq: ------- {} GHz", pcs.cpu_freq());
            println!("CPU Temp: ------- {} * C", pcs.cpu_temp());
            println!("CPU Load: ------- {} %", pcs.cpu_load());
            println!("Memory Used: ---- {} GiB", pcs.memory_used());
            println!("GPU Temp: ------- {} * C", pcs.gpu_temp());
            println!("GPU Core Clock: - {} MHz", pcs.gpu_core_clock());
            println!("GPU Core Load: -- {} %", pcs.gpu_core_load());
            println!("GPU VRAM Clock: - {} MHz", pcs.gpu_memory_clock());
            println!("GPU VRAM Load: -- {} %", pcs.gpu_memory_load());
            println!("");

            println!("Sensors:");
            for (i, c) in pcs.sensors().iter().enumerate() {
                println!("\t{}. {:?}", i + 1, c)
            }

            println!("");
            println!("Probed!");

            Ok(())
        }
        Commands::Commune => {
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
                .timeout(std::time::Duration::from_secs(2))
                .open()
                .map_err(|why| {
                    ExitMsg::new(
                        ExitCode::GenericError,
                        format!("Failed to open serial port, reason: \"{}\".", why),
                    )
                })?;

            serial::serial_task(&mut f)
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
