// Serial communication

use crate::reaction::{InputKey, Peripheral};
use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use std::time::{Duration, Instant};

// Utility
const CMD_GREET: &[u8] = b"\x05\r\n";
const CMD_HEARTBEAT: &[u8] = b"H\r\n";

const CMD_TEST: &[u8] = b"U\x37\r\n";
const CMD_UPDATE: &[u8] = b"U\x38\r\n";
const CMD_DISCONNECT: &[u8] = b"U\x39\r\n";

const CMD_GET_INPUT_KEYS: &[u8] = b"U\x30\r\n";
const CMD_GET_PERIPHERALS: &[u8] = b"U\x31\r\n";
const PERIPHERAL_ID_KEYBOARD: u8 = 0b1000_0000;
const PERIPHERAL_ID_KNOBS_1: u8 = 0b1000_0010;
const PERIPHERAL_ID_KNOBS_2: u8 = 0b1000_0011;
const PERIPHERAL_ID_PEDAL_1: u8 = 0b1000_0101;
const PERIPHERAL_ID_PEDAL_2: u8 = 0b1000_0110;
const PERIPHERAL_ID_PEDAL_3: u8 = 0b1000_0111;

const RSP_HEARTBEAT: &[u8] = b"H\r\n";
const RSP_UNKNOWN: &[u8] = b"?\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n";
// const RSP_DEV1_ACK: &[u8] = b"U\x11\x06\r\n";
// const RSP_DEV2_ACK: &[u8] = b"U\x12\x06\r\n";
// const RSP_DEV3_ACK: &[u8] = b"U\x13\x06\r\n";
const RSP_DEV4_ACK: &[u8] = b"U\x14\x06\r\n";

pub enum SerialCommand {
    GetPeripherals,
    UpdateDevice,
    DisconnectDevice,
    TestFunction,
}

#[derive(PartialEq)]
pub struct SerialConnectionDetails {
    pub firmware_version: String,
    pub device_uid: String,
}

#[derive(PartialEq)]
pub enum SerialEvent {
    Connected(SerialConnectionDetails),
    GetInputKeys(HashSet<InputKey>),
    GetPeripherals(HashSet<Peripheral>),
    LostConnection,
    Disconnected,
}

// TODO: replace ExitMsg with SerialErr (since serial is not used in the main thread)

fn get_serial_string(f: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, ExitMsg> {
    let timeout = Instant::now() + Duration::from_secs(3);
    let mut buf = Vec::new();

    loop {
        if Instant::now() >= timeout {
            return Err(ExitMsg::new(
                ExitCode::SerialReadTimeout,
                "Serial read timeout.".to_owned(),
            ));
        }

        let mut b = [0u8; 1];
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
            // let s = String::from_utf8(buf);
            // if s.is_err() {
            //     return Err(ExitMsg::new(
            //         ExitCode::SerialReadBadData,
            //         "Serial read bad data.".to_owned(),
            //     ));
            // }
            // log::debug!("Serial got string {:?}.", s.clone().unwrap().as_bytes());
            return Ok(buf);
        }
    }
}

fn send_bytes(f: &mut Box<dyn SerialPort>, send: &[u8]) -> Result<(), ExitMsg> {
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

    Ok(())
}

fn expect_string(f: &mut Box<dyn SerialPort>, expect: &[u8]) -> Result<(), ExitMsg> {
    let s = get_serial_string(f).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialExpectRecieveError,
            format!("Failed to get message '{:?}', reason: '{}'.", expect, why),
        )
    })?;

    let matching = s.iter().zip(expect).filter(|&(a, b)| a == b).count() == s.len();

    if !matching {
        // TODO: check if s matches RSP_UNKNOWN
        return Err(ExitMsg::new(
            ExitCode::SerialExpectMatchError,
            format!("Failed to match message '{:?}', got '{:?}'.", expect, s),
        ));
    }

    Ok(())
}

fn send_expect(f: &mut Box<dyn SerialPort>, send: &[u8], expect: &[u8]) -> Result<(), ExitMsg> {
    send_bytes(f, send)?;
    expect_string(f, expect)?;
    Ok(())
}

// Tasks

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<SerialConnectionDetails, ExitMsg> {
    // Host confirms protocol is good, recieves "link established" with some info about the device
    send_bytes(f, CMD_GREET)?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != b'L' {
        todo!()
    }

    let mut firmware_version: Option<_> = None;
    let mut device_uid: Option<_> = None;
    for (i, s) in resp.split(|c| *c == b',').enumerate() {
        if i == 1 {
            firmware_version = Some(s);
        } else if i == 2 {
            device_uid = Some(s);
        }
    }

    if firmware_version.is_none() || device_uid.is_none() {
        todo!()
    }

    Ok(SerialConnectionDetails {
        firmware_version: String::from_utf8(firmware_version.unwrap().to_vec())
            .unwrap_or("UNKNOWN FIRMWARE".to_string()),
        device_uid: String::from_utf8(device_uid.unwrap().to_vec())
            .unwrap_or("UNKNOWN UID".to_string()),
    })
}

fn transmit_heartbeat(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // confirm the device is still alive
    send_expect(f, CMD_HEARTBEAT, RSP_HEARTBEAT)
}

fn transmit_get_input_keys(f: &mut Box<dyn SerialPort>) -> Result<HashSet<InputKey>, ExitMsg> {
    send_bytes(f, CMD_GET_INPUT_KEYS)?;
    let _resp = get_serial_string(f)?;

    let mut result = HashSet::new();
    result.insert(InputKey::KeyboardSwitch1);

    Ok(result)
}

fn transmit_get_peripherals(f: &mut Box<dyn SerialPort>) -> Result<HashSet<Peripheral>, ExitMsg> {
    send_bytes(f, CMD_GET_PERIPHERALS)?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != b'A' {
        todo!()
    }

    let mut result = HashSet::new();
    for c in resp {
        match c {
            PERIPHERAL_ID_KEYBOARD => {
                result.insert(Peripheral::Keyboard);
            }
            PERIPHERAL_ID_KNOBS_1 => {
                result.insert(Peripheral::Knobs1);
            }
            PERIPHERAL_ID_KNOBS_2 => {
                result.insert(Peripheral::Knobs2);
            }
            PERIPHERAL_ID_PEDAL_1 => {
                result.insert(Peripheral::Pedal1);
            }
            PERIPHERAL_ID_PEDAL_2 => {
                result.insert(Peripheral::Pedal2);
            }
            PERIPHERAL_ID_PEDAL_3 => {
                result.insert(Peripheral::Pedal3);
            }
            _ => {}
        }
    }

    Ok(result)
}

fn transmit_update_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // tell the device to reboot for updating
    send_expect(f, CMD_UPDATE, RSP_DISCONNECTED)
}

fn transmit_disconnect_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // tell the device to disconnect cleanly
    send_expect(f, CMD_DISCONNECT, RSP_DISCONNECTED)
}

fn transmit_test_signal(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    send_expect(f, CMD_TEST, RSP_DEV4_ACK)
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
    // Flush serial command queue
    while let Ok(_) = serialcommand_rx.try_recv() {}

    // Greet and link up
    let device_info = greet_host(f)?;
    // TODO: check that firmware version is ok
    serialevent_tx
        .send(SerialEvent::Connected(device_info))
        .expect("failed to send command");

    let peripherals = transmit_get_peripherals(f)?;
    serialevent_tx
        .send(SerialEvent::GetPeripherals(peripherals))
        .expect("failed to send command");

    let mut timer = Instant::now();
    'forv: loop {
        // TODO: Despite yielding, this can still lead to high CPU usage, and should probably be fixed.
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now() + Duration::from_millis(5);

        transmit_heartbeat(f)?; // TODO: replace with get input keys

        // TODO: query device for pressed buttons
        // let keys = transmit_get_input_keys(f)?;
        // if let Err(_e) = serialevent_tx.send(SerialEvent::GetInputKeys(keys)) {
        //     todo!();
        // }

        while let Ok(cmd) = serialcommand_rx.try_recv() {
            match cmd {
                SerialCommand::GetPeripherals => {
                    let peripherals = transmit_get_peripherals(f)?;
                    if let Err(_e) = serialevent_tx.send(SerialEvent::GetPeripherals(peripherals)) {
                        todo!();
                    }
                }
                SerialCommand::UpdateDevice => {
                    transmit_update_signal(f)?;
                    if let Err(e) = serialevent_tx.send(SerialEvent::Disconnected) {
                        log::warn!("Disconnect event signal failed, reason: `{}`", e);
                    }
                    break 'forv; // The device has disconnected, we should too.
                }
                SerialCommand::DisconnectDevice => {
                    transmit_disconnect_signal(f)?;
                    if let Err(e) = serialevent_tx.send(SerialEvent::Disconnected) {
                        log::warn!("Disconnect event signal failed, reason: `{}`", e);
                    }
                    break 'forv; // The device has disconnected, we should too.
                }
                SerialCommand::TestFunction => {
                    transmit_test_signal(f)?;
                }
            }
        }
    }

    Ok(())
}
