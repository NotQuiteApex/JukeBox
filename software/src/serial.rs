// Serial communication

use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use std::time::{Duration, Instant};

// Utility
const CMD_GREET: &[u8] = b"JB\x05\r\n";
const CMD_PROTOCOL_ACCEPT: &[u8] = b"P\x06\r\n";
const CMD_HEARTBEAT: &[u8] = b"H\x30\r\n";
const CMD_DISCONNECT: &[u8] = b"U\x30\r\n";
const CMD_UPDATE: &[u8] = b"U\x31\r\n";
const CMD_TEST_FUNC_0: &[u8] = b"U\x32\x30\r\n";

const RSP_PROTOCOL: &str = "P001\r\n";
const RSP_LINK_ESTABLISHED: &str = "L\x06\r\n";
const RSP_HEARTBEAT: &str = "H\x31\r\n";
const RSP_DISCONNECTED: &str = "\x04\x04\r\n";
const RSP_DEV1_ACK: &str = "U\x11\x06\r\n";
const RSP_DEV2_ACK: &str = "U\x12\x06\r\n";
const RSP_DEV3_ACK: &str = "U\x13\x06\r\n";
const RSP_DEV4_ACK: &str = "U\x14\x06\r\n";

pub enum SerialCommand {
    UpdateDevice,
    DisconnectDevice,
    RefreshPeripherals,
    TestFunction0,
}

pub enum SerialEvent {
    Connected,
    LostConnection,
    Disconnected,
}

pub enum JukeBoxPeripherals {
    Keyboard,
    Knobs1,
    Knobs2,
    Pedal1,
    Pedal2,
    Pedal3,
}

fn get_serial_string(f: &mut Box<dyn SerialPort>) -> Result<String, ExitMsg> {
    let timeout = Instant::now() + Duration::from_secs(3);
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
    // Sends greeting, recieves protocol string
    send_expect(f, CMD_GREET, RSP_PROTOCOL)
    // TODO: send nack in response to bad protocol
}

fn link_confirm_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Host confirms protocol is good, recieves "link established"
    send_expect(f, CMD_PROTOCOL_ACCEPT, RSP_LINK_ESTABLISHED)
}

fn transmit_heartbeat(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // confirm the device is still alive
    send_expect(f, CMD_HEARTBEAT, RSP_HEARTBEAT)
}

fn transmit_disconnect_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // tell the device to disconnect cleanly
    send_expect(f, CMD_DISCONNECT, RSP_DISCONNECTED)
}

fn transmit_update_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // tell the device to reboot for updating
    send_expect(f, CMD_UPDATE, RSP_DISCONNECTED)
}

fn transmit_test_function_0_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, CMD_TEST_FUNC_0, RSP_DEV2_ACK)
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
    serialcommand_rx: &Receiver<SerialCommand>,
    serialevent_tx: &Sender<SerialEvent>,
) -> Result<(), ExitMsg> {
    greet_host(f)?;
    link_confirm_host(f)?;

    serialevent_tx
        .send(SerialEvent::Connected)
        .expect("failed to send command");

    let mut timer = Instant::now();

    'forv: loop {
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now() + Duration::from_millis(500);

        transmit_heartbeat(f)?;

        // TODO: query device for pressed buttons

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
                _ => todo!(),
            }
        }

        // TODO: implement recieving command check
    }

    Ok(())
}
