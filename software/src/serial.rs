// Serial communication

use crate::system::SystemReport;
use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::ops::Add;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use std::time::{Duration, Instant};

// Utility

const RSP_DISCONNECTED: &str = "\x04\x04\r\n";

pub enum SerialCommand {
    UpdateDevice,
    DisconnectDevice,
    TestFunction0,
}

pub enum SerialEvent {
    Connected,
    LostConnection,
    Disconnected,
}

fn get_serial_string(f: &mut Box<dyn SerialPort>) -> Result<String, ExitMsg> {
    let timeout = Instant::now().add(Duration::from_secs(3));
    let mut buf = Vec::new();

    loop {
        if Instant::now() >= timeout {
            return Err(ExitMsg::new(
                ExitCode::SerialReadTimeout,
                "Serial read timeout.".to_owned(),
            ));
        }

        let mut b: [u8; 1] = [0; 1];
        let res = f.read(&mut b);
        if res.is_err() {
            continue;
        }
        buf.push(b[0]);

        let len = buf.len();
        if len >= 2
            && buf.get(len - 2).map_or(false, |v| v == &b'\r')
            && buf.get(len - 1).map_or(false, |v| v == &b'\n')
        {
            // we matched the string, return
            let s = String::from_utf8(buf);
            if s.is_err() {
                return Err(ExitMsg::new(
                    ExitCode::SerialReadBadData,
                    "Serial read bad data.".to_owned(),
                ));
            }
            log::debug!("Serial got string {:?}.", s.clone().unwrap().as_bytes());
            return Ok(s.unwrap());
        }
    }
}

fn send_expect(f: &mut Box<dyn SerialPort>, send: &[u8], expect: &str) -> Result<(), ExitMsg> {
    f.write_all(send).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialSendMessageError,
            format!("Failed to send message '{:?}', reason: '{}'.", send, why),
        )
    })?;
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialSendFlushError,
            format!("Failed to flush message '{:?}', reason: '{}'.", send, why),
        )
    })?;

    let s = get_serial_string(f).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialExpectRecieveError,
            format!("Failed to get message '{:?}', reason: '{}'.", expect, why),
        )
    })?;
    if s != String::from(expect) {
        return Err(ExitMsg::new(
            ExitCode::SerialExpectMatchError,
            format!("Failed to match message '{:?}', got '{}'.", expect, s),
        ));
    }

    Ok(())
}

// Tasks

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"JB\x05\r\n", "P001\r\n")
    // TODO: send nack in response to bad protocol
}

fn link_confirm_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"P\x06\r\n", "L\x06\r\n")
}

fn transmit_heartbeat(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"H\x30\r\n", "H\x31\r\n")
}

fn transmit_system_info(f: &mut Box<dyn SerialPort>, pcs: &SystemReport) -> Result<(), ExitMsg> {
    let m = format!(
        "D\x11\x30{}\x1F{}\x1F{}\x1F\r\n",
        pcs.cpu_name, pcs.gpu_name, pcs.memory_total,
    );
    let m = m.as_bytes();
    if m.len() > 128 {
        log::warn!("TInit string longer than 64 bytes!");
        log::warn!("TInit CPU Brand: '{}'.", pcs.cpu_name);
        log::warn!("TInit GPU Brand: '{}'.", pcs.gpu_name);
        log::warn!("TInit Memory: '{}'.", pcs.memory_total);
        log::warn!("TInit string len: {} bytes.", m.len());
        // log::warn!("TInit string: {:?}", m);
    }

    send_expect(f, m, "D\x11\x06\r\n")
}

fn transmit_system_stats(f: &mut Box<dyn SerialPort>, pcs: &SystemReport) -> Result<(), ExitMsg> {
    let m = format!(
        "D\x11\x31{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
        pcs.cpu_freq,
        pcs.cpu_temp,
        pcs.cpu_load,
        pcs.memory_used,
        pcs.gpu_temp,
        pcs.gpu_core_clock,
        pcs.gpu_core_load,
        pcs.gpu_memory_clock,
        pcs.gpu_memory_load,
    );
    let m = m.as_bytes();

    send_expect(f, m, "D\x11\x06\r\n")
}

fn transmit_disconnect_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"U\x30\r\n", RSP_DISCONNECTED)
}

fn transmit_update_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"U\x31\r\n", RSP_DISCONNECTED)
}

fn transmit_test_function_0_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, b"U\x32\x30\r\n", "U\x12\x06\r\n")
}

pub fn serial_get_device() -> Result<Box<dyn SerialPort>, ExitMsg> {
    let ports = serialport::available_ports().map_err(|why| {
        ExitMsg::new(
            ExitCode::GenericError,
            format!("Failed to enumerate serial ports, reason: \"{}\".", why),
        )
    })?;
    let ports: Vec<_> = ports
        .iter()
        .filter(|p| match &p.port_type {
            serialport::SerialPortType::UsbPort(p) => p.pid == 0xF20A && p.vid == 0x1209,
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

    Ok(serialport::new(port.port_name.clone(), 115200)
        .timeout(std::time::Duration::from_millis(10))
        .open()
        .map_err(|why| {
            ExitMsg::new(
                ExitCode::GenericError,
                format!("Failed to open serial port, reason: \"{}\".", why),
            )
        })?)
}

pub fn serial_task(
    f: &mut Box<dyn SerialPort>,
    sysreport_rx: &Receiver<SystemReport>,
    serialcommand_rx: &Receiver<SerialCommand>,
    serialevent_tx: &Sender<SerialEvent>,
) -> Result<(), ExitMsg> {
    // let mut pcs = PCSystem::new()?;
    let mut sysreport = sysreport_rx
        .recv()
        .expect("did not recieve sysreport for serial");

    greet_host(f)?;
    link_confirm_host(f)?;

    serialevent_tx
        .send(SerialEvent::Connected)
        .expect("failed to send command");

    transmit_system_info(f, &sysreport)?;

    let mut timer = Instant::now();

    'forv: loop {
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now().add(Duration::from_millis(500));

        transmit_heartbeat(f)?;

        if let Ok(newreport) = sysreport_rx.try_recv() {
            sysreport = newreport;
            transmit_system_stats(f, &sysreport)?;
        }

        while let Ok(cmd) = serialcommand_rx.try_recv() {
            match cmd {
                SerialCommand::TestFunction0 => {
                    transmit_test_function_0_signal(f)?;
                }
                SerialCommand::DisconnectDevice => {
                    transmit_disconnect_signal(f)?;
                    if let Err(e) = serialevent_tx.send(SerialEvent::Disconnected) {
                        log::warn!("Disconnect event signal failed, reason: `{}`", e);
                    }
                    break 'forv; // The device has disconnected, we should too.
                }
                SerialCommand::UpdateDevice => {
                    transmit_update_signal(f)?;
                    if let Err(e) = serialevent_tx.send(SerialEvent::Disconnected) {
                        log::warn!("Disconnect event signal failed, reason: `{}`", e);
                    }
                    break 'forv; // The device has disconnected, we should too.
                }
            }
        }

        // TODO: implement recieving command check
    }

    Ok(())
}
