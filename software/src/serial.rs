// Serial communication

use crate::reaction::{InputKey, Peripheral};

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serialport::SerialPort;

// Utility
const CMD_GREET: &[u8] = b"\x05\r\n";
const CMD_NEGATIVE_ACK: &[u8] = b"\x15\r\n";

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

const RSP_UNKNOWN: &[u8] = b"?\r\n\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n\r\n";
// const RSP_DEV1_ACK: &[u8] = b"U\x11\x06\r\n";
// const RSP_DEV2_ACK: &[u8] = b"U\x12\x06\r\n";
// const RSP_DEV3_ACK: &[u8] = b"U\x13\x06\r\n";
const RSP_DEV4_ACK: &[u8] = b"U\x14\x06\r\n\r\n";

// #[derive(PartialEq, Debug)]
// pub enum SerialErr {
//     FailedToScanSerialPorts,
//     FailedToFindSerialPort,
//     FailedToOpenSerialPort,

//     FailedToWriteMessage,
//     FailedToFlushMessage,

//     SerialReadTimeout,
//     SerialExpectMismatch,
//     SerialDeviceDidNotUnderstand,

//     FailedToSendDeviceInfo,
//     FailedToSendPeripheralInfo,
//     FailedToSendInputInfo,
//     FailedToSendDisconnectInfo,

//     FailedToParseDeviceInfo,
//     FailedToParsePeripheralInfo,
//     FailedToParseInputInfo,
// }

pub enum SerialCommand {
    GetPeripherals,
    UpdateDevice,
    DisconnectDevice,
    TestFunction,
}

#[derive(PartialEq, Clone)]
pub struct SerialConnectionDetails {
    pub firmware_version: String,
    pub device_uid: String,
}

#[derive(PartialEq, Clone)]
pub enum SerialEvent {
    Connected(SerialConnectionDetails),
    GetInputKeys(HashSet<InputKey>),
    GetPeripherals(HashSet<Peripheral>),
    LostConnection,
    Disconnected,
}

fn get_serial_string(f: &mut Box<dyn SerialPort>) -> Result<Vec<u8>> {
    let timeout = Instant::now() + Duration::from_secs(3);
    let mut buf = Vec::new();

    loop {
        if Instant::now() >= timeout {
            bail!("read timed out");
        }

        let mut b = [0u8; 1];
        let res = f.read(&mut b);
        if res.is_err() {
            continue;
        }
        buf.push(b[0]);

        let len = buf.len();
        if len > 4
            && buf.get(len - 4).map_or(false, |v| v == &b'\r')
            && buf.get(len - 3).map_or(false, |v| v == &b'\n')
            && buf.get(len - 2).map_or(false, |v| v == &b'\r')
            && buf.get(len - 1).map_or(false, |v| v == &b'\n')
        {
            return Ok(buf);
        }
    }
}

fn send_bytes(f: &mut Box<dyn SerialPort>, bytes: &[u8]) -> Result<()> {
    f.write_all(bytes)
        .with_context(|| format!("failed to write message {:?}", bytes))?;
    f.flush().context("failed to flush message")?;

    Ok(())
}

fn expect_string(f: &mut Box<dyn SerialPort>, expect: &[u8]) -> Result<()> {
    let s = get_serial_string(f)?;

    let matching = s.iter().zip(expect).filter(|&(a, b)| a == b).count() == s.len();

    if !matching {
        // TODO: check if s matches RSP_UNKNOWN
        let matches_unknown = s.iter().zip(RSP_UNKNOWN).filter(|&(a, b)| a == b).count() == s.len();
        if matches_unknown {
            send_negative_ack(f)?;
            bail!("device did not understand command");
        }

        bail!("expect mismatch (expected {:?}, got {:?}", expect, s);
    }

    Ok(())
}

fn send_expect(f: &mut Box<dyn SerialPort>, send: &[u8], expect: &[u8]) -> Result<()> {
    send_bytes(f, send).with_context(|| format!("failed to send bytes {:?}", send))?;
    expect_string(f, expect).with_context(|| format!("failed to get bytes {:?}", expect))?;
    Ok(())
}

// Tasks

fn send_negative_ack(f: &mut Box<dyn SerialPort>) -> Result<()> {
    send_bytes(f, CMD_NEGATIVE_ACK).context("failed to send nack")?;
    Ok(())
}

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<SerialConnectionDetails> {
    // Host confirms protocol is good, recieves "link established" with some info about the device
    send_bytes(f, CMD_GREET)?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != b'L' {
        send_negative_ack(f)?;
        bail!("failed to parse device info (command character mismatch)");
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
        send_negative_ack(f)?;
        bail!("failed to parse device info (missing firmware version or device uid)");
    }

    let firmware_version = match String::from_utf8(firmware_version.unwrap().to_vec()) {
        Ok(s) => s,
        Err(_) => {
            send_negative_ack(f)?;
            bail!("failed to parse device info (failed to convert firmware version to utf-8)");
        }
    };
    let device_uid = match String::from_utf8(device_uid.unwrap().to_vec()) {
        Ok(s) => s,
        Err(_) => {
            send_negative_ack(f)?;
            bail!("failed to parse device info (failed to convert device uid to utf-8)");
        }
    };

    Ok(SerialConnectionDetails {
        firmware_version: firmware_version,
        device_uid: device_uid,
    })
}

fn transmit_get_input_keys(f: &mut Box<dyn SerialPort>) -> Result<HashSet<InputKey>> {
    send_bytes(f, CMD_GET_INPUT_KEYS)?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != b'I' {
        send_negative_ack(f)?;
        bail!("failed to parse input keys (command character mismatch)");
    }

    let mut result = HashSet::new();
    let mut i = resp.iter();
    loop {
        match i.next() {
            Some(c) => match *c {
                PERIPHERAL_ID_KEYBOARD => {
                    let w2 = i.next();
                    let w1 = i.next();
                    if w2.is_none() || w1.is_none() {
                        bail!("failed to parse input keys (missing keyboard words)");
                    }
                    result.extend(InputKey::decode_keyboard(*w2.unwrap(), *w1.unwrap()));
                }
                PERIPHERAL_ID_KNOBS_1 => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing knob 1 word)");
                    }
                    result.extend(InputKey::decode_knobs1(*w.unwrap()));
                }
                PERIPHERAL_ID_KNOBS_2 => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing knob 2 word)");
                    }
                    result.extend(InputKey::decode_knobs2(*w.unwrap()));
                }
                PERIPHERAL_ID_PEDAL_1 => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing pedal 1 word)");
                    }
                    result.extend(InputKey::decode_pedal1(*w.unwrap()));
                }
                PERIPHERAL_ID_PEDAL_2 => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing pedal 2 word)");
                    }
                    result.extend(InputKey::decode_pedal2(*w.unwrap()));
                }
                PERIPHERAL_ID_PEDAL_3 => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing pedal 3 word)");
                    }
                    result.extend(InputKey::decode_pedal3(*w.unwrap()));
                }
                _ => {}
            },
            None => break,
        }
    }

    Ok(result)
}

fn transmit_get_peripherals(f: &mut Box<dyn SerialPort>) -> Result<HashSet<Peripheral>> {
    send_bytes(f, CMD_GET_PERIPHERALS)?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != b'A' {
        send_negative_ack(f)?;
        bail!("failed to parse get peripherals (command character mismatch)");
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

fn transmit_update_signal(f: &mut Box<dyn SerialPort>) -> Result<()> {
    // tell the device to reboot for updating
    send_expect(f, CMD_UPDATE, RSP_DISCONNECTED)
}

fn transmit_disconnect_signal(f: &mut Box<dyn SerialPort>) -> Result<()> {
    // tell the device to disconnect cleanly
    send_expect(f, CMD_DISCONNECT, RSP_DISCONNECTED)
}

fn transmit_test_signal(f: &mut Box<dyn SerialPort>) -> Result<()> {
    send_expect(f, CMD_TEST, RSP_DEV4_ACK)
}

pub fn serial_get_device() -> Result<Box<dyn SerialPort>> {
    let ports = serialport::available_ports().context("failed to scan serial ports")?;
    let ports: Vec<_> = ports
        .iter()
        .filter(|p| match &p.port_type {
            serialport::SerialPortType::UsbPort(p) => p.pid == 0xF20A && p.vid == 0x1209,
            _ => false,
        })
        .collect();

    if ports.len() == 0 {
        bail!("failed to find any jukebox serial ports");
    }

    // TODO: provide an argument to choose from this vector
    let port = ports.get(0).unwrap();

    Ok(serialport::new(port.port_name.clone(), 115200)
        .timeout(std::time::Duration::from_millis(10))
        .open()
        .context("failed to open serial port")?)
}

pub fn serial_comms(
    f: &mut Box<dyn SerialPort>,
    serialcommand_rx: &Receiver<SerialCommand>,
    serialevent_tx: &Sender<SerialEvent>,
) -> Result<()> {
    // Flush serial command queue
    while let Ok(_) = serialcommand_rx.try_recv() {}

    // Greet and link up
    let device_info = greet_host(f)?;
    // TODO: check that firmware version is ok
    serialevent_tx
        .send(SerialEvent::Connected(device_info))
        .context("failed to send device info")?;

    let peripherals = transmit_get_peripherals(f)?;
    serialevent_tx
        .send(SerialEvent::GetPeripherals(peripherals))
        .context("failed to send peripheral info")?;

    let mut timer = Instant::now();
    'forv: loop {
        // TODO: Despite yielding, this can still lead to high CPU usage, and should probably be fixed.
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now() + Duration::from_millis(25);

        let keys = transmit_get_input_keys(f)?;
        // log::info!("keys {:?}", keys);
        serialevent_tx
            .send(SerialEvent::GetInputKeys(keys))
            .context("failed to send input info")?;

        while let Ok(cmd) = serialcommand_rx.try_recv() {
            match cmd {
                SerialCommand::GetPeripherals => {
                    let peripherals = transmit_get_peripherals(f)?;
                    serialevent_tx
                        .send(SerialEvent::GetPeripherals(peripherals))
                        .context("failed to send peripheral info")?;
                }
                SerialCommand::UpdateDevice => {
                    transmit_update_signal(f)?;
                    serialevent_tx
                        .send(SerialEvent::Disconnected)
                        .context("failed to send disconnect info")?;
                    break 'forv; // The device has disconnected, we should too.
                }
                SerialCommand::DisconnectDevice => {
                    transmit_disconnect_signal(f)?;
                    serialevent_tx
                        .send(SerialEvent::Disconnected)
                        .context("failed to send disconnect info")?;
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

pub fn serial_task(
    brkr: Arc<AtomicBool>,
    s_cmd_rx: Receiver<SerialCommand>,
    s_evnt_tx: Sender<SerialEvent>,
) -> Result<()> {
    // TODO: check application cpu usage when device is connected
    loop {
        if brkr.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let mut f = match serial_get_device() {
            Err(_) => {
                sleep(Duration::from_secs(1));
                continue;
            }
            Ok(f) => f,
        };

        match serial_comms(&mut f, &s_cmd_rx, &s_evnt_tx) {
            Err(e) => {
                log::warn!("Serial device error: {:?}", e);
                s_evnt_tx
                    .send(SerialEvent::LostConnection)
                    .context("failed to send lost connection")?;
            }
            Ok(_) => log::info!("Serial device successfully disconnected. Looping..."),
        };
    }

    Ok(())
}
