// Serial communication

use crate::reaction::InputKey;

use std::collections::HashSet;
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use jukebox_util::peripheral::{
    KeyInputs, KnobInputs, PedalInputs, IDENT_KEY_INPUT, IDENT_KNOB_INPUT, IDENT_PEDAL_INPUT,
    IDENT_UNKNOWN_INPUT,
};
use jukebox_util::protocol::{
    CMD_DISCONNECT, CMD_END, CMD_GET_INPUT_KEYS, CMD_GREET, CMD_NEGATIVE_ACK, CMD_UPDATE,
    RSP_DISCONNECTED, RSP_END, RSP_INPUT_HEADER, RSP_LINK_DELIMITER, RSP_LINK_HEADER, RSP_UNKNOWN,
};
use serialport::SerialPort;

#[derive(PartialEq, Clone)]
pub struct SerialConnectionDetails {
    pub input_identifier: u8,
    pub firmware_version: String,
    pub device_uid: String,
}

pub enum SerialCommand {
    // GetPeripherals,
    UpdateDevice,
    DisconnectDevice,
    // TestFunction,
}

#[derive(PartialEq, Clone)]
pub enum SerialEvent {
    Connected(SerialConnectionDetails),
    GetInputKeys(HashSet<InputKey>),
    // GetPeripherals(HashSet<Peripheral>),
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

        if buf.len() > RSP_END.len() {
            let s = &buf[(buf.len() - RSP_END.len())..buf.len()];
            let c = s.iter().zip(RSP_END).all(|(a, b)| a == b);
            if c {
                return Ok(buf);
            }
        }
    }
}

fn send_cmd(f: &mut Box<dyn SerialPort>, c: u8) -> Result<()> {
    let mut cmd = vec![c];
    cmd.extend_from_slice(CMD_END);
    send_bytes(f, cmd.as_slice()).with_context(|| format!("failed to send cmd {}", c))
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
        let matches_unknown = s
            .iter()
            .zip([RSP_UNKNOWN].iter().chain(RSP_END).collect::<Vec<_>>())
            .filter(|&(a, b)| a == b)
            .count()
            == s.len();
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
    send_cmd(f, CMD_NEGATIVE_ACK).context("failed to send nack")?;
    Ok(())
}

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<SerialConnectionDetails> {
    // Host confirms protocol is good, recieves "link established" with some info about the device
    send_cmd(f, CMD_GREET).context("failed to send greet")?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != RSP_LINK_HEADER {
        send_negative_ack(f)?;
        bail!("failed to parse device info (command character mismatch)");
    }

    let mut input_identifier = None;
    let mut firmware_version = None;
    let mut device_uid = None;
    for (i, s) in resp.split(|c| *c == RSP_LINK_DELIMITER).enumerate() {
        if i == 1 {
            input_identifier = Some(s.get(0).unwrap_or(&IDENT_UNKNOWN_INPUT));
        } else if i == 2 {
            firmware_version = Some(s);
        } else if i == 3 {
            device_uid = Some(s);
        }
    }

    if input_identifier.is_none() || firmware_version.is_none() || device_uid.is_none() {
        send_negative_ack(f)?;
        bail!("failed to parse device info (missing input identifier, firmware version, or device uid)");
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
        input_identifier: *input_identifier.unwrap(),
        firmware_version: firmware_version,
        device_uid: device_uid,
    })
}

fn transmit_get_input_keys(f: &mut Box<dyn SerialPort>) -> Result<HashSet<InputKey>> {
    send_cmd(f, CMD_GET_INPUT_KEYS).context("failed to send get input keys")?;
    let resp = get_serial_string(f)?;

    if *resp.iter().nth(0).unwrap_or(&0) != RSP_INPUT_HEADER {
        log::info!("rsp input: {:?}", resp);
        send_negative_ack(f)?;
        bail!("failed to parse input keys (command character mismatch)");
    }

    let mut result = HashSet::new();
    let mut i = resp.iter();
    loop {
        match i.next() {
            Some(c) => match *c {
                IDENT_KEY_INPUT => {
                    let w2 = i.next();
                    let w1 = i.next();
                    if w2.is_none() || w1.is_none() {
                        bail!("failed to parse input keys (missing keyboard words)");
                    }
                    let keypad = KeyInputs::decode(&[*c, *w2.unwrap(), *w1.unwrap()])
                        .map_err(|_| anyhow!("failed to decode key inputs"))?;
                    result.extend(InputKey::trans_keys(keypad));
                }
                IDENT_KNOB_INPUT => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing knob 1 word)");
                    }
                    let knobpad = KnobInputs::decode(&[*c, *w.unwrap()])
                        .map_err(|_| anyhow!("failed to decode knob inputs"))?;
                    result.extend(InputKey::trans_knob(knobpad));
                }
                IDENT_PEDAL_INPUT => {
                    let w = i.next();
                    if w.is_none() {
                        bail!("failed to parse input keys (missing pedal 1 word)");
                    }
                    let pedalpad = PedalInputs::decode(&[*c, *w.unwrap()])
                        .map_err(|_| anyhow!("failed to decode pedal inputs"))?;
                    result.extend(InputKey::trans_pedals(pedalpad));
                }
                _ => {}
            },
            None => break,
        }
    }

    Ok(result)
}

fn transmit_update_signal(f: &mut Box<dyn SerialPort>) -> Result<()> {
    // tell the device to reboot for updating
    let mut cmd = vec![CMD_UPDATE];
    cmd.extend_from_slice(CMD_END);
    let mut rsp = vec![RSP_DISCONNECTED];
    rsp.extend_from_slice(RSP_END);

    send_expect(f, &cmd, &rsp)
}

fn transmit_disconnect_signal(f: &mut Box<dyn SerialPort>) -> Result<()> {
    // tell the device to disconnect cleanly
    let mut cmd = vec![CMD_DISCONNECT];
    cmd.extend_from_slice(CMD_END);
    let mut rsp = vec![RSP_DISCONNECTED];
    rsp.extend_from_slice(RSP_END);

    send_expect(f, &cmd, &rsp)
}

pub fn serial_get_device() -> Result<Box<dyn SerialPort>> {
    let ports = serialport::available_ports().context("failed to scan serial ports")?;
    let ports: Vec<_> = ports
        .iter()
        .filter(|p| match &p.port_type {
            serialport::SerialPortType::UsbPort(p) => p.vid == 0x1209 && p.pid == 0xF20A,
            _ => false,
        })
        .collect();

    log::debug!("serial ports found: {:?}", ports);

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

    let mut timer = Instant::now();
    'forv: loop {
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now() + Duration::from_millis(25);

        let keys = transmit_get_input_keys(f)?;
        serialevent_tx
            .send(SerialEvent::GetInputKeys(keys))
            .context("failed to send input info")?;

        while let Ok(cmd) = serialcommand_rx.try_recv() {
            match cmd {
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
            Err(e) => {
                log::debug!("get_serial_device() failure: {:#}", e);
                sleep(Duration::from_secs(1));
                continue;
            }
            Ok(f) => f,
        };

        match serial_comms(&mut f, &s_cmd_rx, &s_evnt_tx) {
            Err(e) => {
                log::warn!("Serial device error: {:#}", e);
                s_evnt_tx
                    .send(SerialEvent::LostConnection)
                    .context("failed to send lost connection")?;
            }
            Ok(_) => log::info!("Serial device successfully disconnected. Looping..."),
        };
    }

    Ok(())
}
