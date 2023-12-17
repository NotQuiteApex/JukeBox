// Serial communication

use crate::system::PCSystem;
use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::ops::Add;
use std::time::{Duration, Instant};

// Utility

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
            // println!("Failed to read byte from serial.");
            // println!("{:?}", res);
            // return Err(());
            continue;
        }
        buf.push(b[0]);

        let len = buf.len();
        // println!("{:?}", buf);
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

// Stages

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Send enquire message to JukeBox
    log::debug!("SerialStage: Greeting");

    if f.write_all(b"JB\x05\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageGreetHost,
            "Failed to send host greeting.".to_owned(),
        ));
    }
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialStageGreetHost,
            format!("Failed to flush string, reason: '{}'.", why),
        )
    })?;

    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("P001\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialStageGreetDevice,
            "Failed to confirm protocol.".to_owned(),
        ));
        // TODO: send nack in response to bad protocol
    }

    Ok(())
}

fn link_confirm_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    log::debug!("SerialStage: LinkConfirmation");

    if f.write_all(b"P\x06\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageLinkConfirmHost,
            "Failed to send protocol ack.".to_owned(),
        ));
    }
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialStageLinkConfirmHost,
            format!("Failed to flush string, reason: '{}'.", why),
        )
    })?;

    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("L\x06\r\n") {
        // response was bad, go to error state.
        return Err(ExitMsg::new(
            ExitCode::SerialStageLinkConfirmDevice,
            "Failed to send link confirmation.".to_owned(),
        ));
    }

    Ok(())
}

// Tasks

fn transmit_tasks_init(f: &mut Box<dyn SerialPort>, pcs: &PCSystem) -> Result<(), ExitMsg> {
    let m = format!(
        "D\x11\x30{}\x1F{}\x1F{}\x1F\r\n",
        pcs.cpu_name(),
        pcs.gpu_name(),
        pcs.memory_total(),
    );
    let m = m.as_bytes();
    if m.len() > 128 {
        log::warn!("TInit string longer than 64 bytes!");
        log::warn!("TInit CPU Brand: '{}'.", pcs.cpu_name());
        log::warn!("TInit GPU Brand: '{}'.", pcs.gpu_name());
        log::warn!("TInit Memory: '{}'.", pcs.memory_total());
        log::warn!("TInit string len: {} bytes.", m.len());
        // log::warn!("TInit string: {:?}", m);
    }

    f.write_all(m).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitSend,
            format!("Failed to send computer part info, reason '{}'.", why),
        )
    })?;
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitSend,
            format!("Failed to flush string, reason: '{}'.", why),
        )
    })?;

    let s = get_serial_string(f).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitAck,
            format!("Failed to get computer part info ack'd, reason '{}'.", why),
        )
    })?;
    if s != String::from("D\x11\x06\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitAck,
            format!("Computer part info ack incorrect, got '{}'.", s),
        ));
    }

    Ok(())
}

fn transmit_tasks_loop(f: &mut Box<dyn SerialPort>, pcs: &PCSystem) -> Result<bool, ExitMsg> {
    f.write_all(b"H\x30\r\n").map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatSend,
            format!("Failed to send heartbeat, reason: '{}'.", why),
        )
    })?;
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatSend,
            format!("Failed to flush string, reason: '{}'.", why),
        )
    })?;

    let s = get_serial_string(f).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatAck,
            format!("Failed to get heartbeat ack'd, reason: '{}'.", why),
        )
    })?;
    if s != String::from("H\x31\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatAck,
            format!("Heartbeat ack incorrect, got '{}'.", s),
        ));
    }

    // pcs.probe_report();

    let m = format!(
        "D\x11\x31{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
        pcs.cpu_freq(),
        pcs.cpu_temp(),
        pcs.cpu_load(),
        pcs.memory_used(),
        pcs.gpu_temp(),
        pcs.gpu_core_clock(),
        pcs.gpu_core_load(),
        pcs.gpu_memory_clock(),
        pcs.gpu_memory_load(),
    );
    let m = m.as_bytes();

    f.write_all(m).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatSend,
            format!("Failed to send computer part stats, reason: '{}'.", why),
        )
    })?;
    f.flush().map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatSend,
            format!("Failed to flush string, reason: '{}'.", why),
        )
    })?;

    let s = get_serial_string(f).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatAck,
            format!(
                "Failed to get computer part stats ack'd, reason: '{}'.",
                why
            ),
        )
    })?;
    if s != String::from("D\x11\x06\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatAck,
            format!("Computer part stats ack incorrect, got: '{}'.", s),
        ));
    }

    // TODO: include a clause for closing the connection and returning true.

    Ok(false)
}

pub fn serial_task(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    let mut pcs = PCSystem::new()?;

    greet_host(f)?;
    link_confirm_host(f)?;

    transmit_tasks_init(f, &pcs)?;

    let mut timer = Instant::now();

    loop {
        if Instant::now() < timer {
            continue;
        }
        timer = Instant::now().add(Duration::from_millis(1000));

        pcs.update();

        if transmit_tasks_loop(f, &pcs)? {
            break;
        }
    }

    Ok(())
}
