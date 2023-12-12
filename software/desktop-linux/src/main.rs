// A lightweight program to simulate JukeBox serial communication.

mod serial;
mod util;

use serialport::SerialPortType;

use crate::util::{ExitCode, ExitMsg};

use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::{cuda_driver_version_major, cuda_driver_version_minor, Nvml};
use pretty_bytes::converter::convert;

fn nvtest() -> Result<(), NvmlError> {
    let nvml = Nvml::init()?;

    let device_count = nvml.device_count()?;
    println!("NVIDIA GPU Devices:");
    for i in 0..device_count {
        let device = nvml.device_by_index(i)?;

        let name = device.name()?;
        let temp = device.temperature(TemperatureSensor::Gpu)?;
        let gfx_clock = device.clock_info(Clock::Graphics)?;
        let mem_clock = device.clock_info(Clock::Memory)?;
        let utils = device.utilization_rates()?;

        println!(
            "{}. {}: {}*C, {} MHz, {} MHz, {} %, {} %",
            i+1,
            name,
            temp,
            gfx_clock,
            mem_clock,
            utils.gpu,
            utils.memory,
        )
    }

    Ok(())
}

fn deffered_main() -> Result<(), ExitMsg> {
    nvtest().map_err(|why|
        ExitMsg::new(
            ExitCode::GenericError,
            format!("Failed to run nvtest: {}", why)
        )
    )?;
    return Ok(());

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
