// Serial communication

use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::ops::Add;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

// Utility

fn get_serial_string(f: &mut Box<dyn SerialPort>) -> Result<String, ExitMsg> {
    let timeout = SystemTime::now().add(Duration::from_secs(3));
    let mut buf = Vec::new();

    loop {
        if SystemTime::now() >= timeout {
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
            log::trace!("Serial got string '{}'.", s.clone().unwrap());
            return Ok(s.unwrap());
        }
    }
}

// Stages

fn serial_greet_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Send enquire message to JukeBox
    log::trace!("SerialStage: GreetHost");

    if f.write_all(b"JB\x05\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageGreetHost,
            "Failed to send host greeting.".to_owned(),
        ));
    }

    Ok(())
}

fn serial_greet_device(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Recieve response, which should contain protocol version.
    log::trace!("SerialStage: GreetDevice");

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

fn serial_link_confirm_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    log::trace!("SerialStage: LinkConfirmHost");

    if f.write_all(b"P\x06\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageLinkConfirmHost,
            "Failed to send protocol ack.".to_owned(),
        ));
    }

    Ok(())
}

fn serial_link_confirm_device(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    log::trace!("SerialStage: LinkConfirmDevice");

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

fn serial_transmit(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    transmit_tasks_init(f)?;

    loop {
        let exit = transmit_tasks_loop(f)?;
        if exit {
            break;
        }
    }

    Ok(())
}

// Tasks

fn transmit_tasks_init(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    let m = format!(
        "D\x11\x30{}\x1F{}\x1F{}GB\x1F\r\n",
        "a0", //"TestCPU",
        "a1", //"TestGPU",
        "a2", //"0",
    );
    let m = m.as_bytes();

    if f.write_all(m).is_err() {
        println!("Error: Failed to send computer part info.");
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitSend,
            "Failed to send computer part info.".to_owned(),
        ));
    }

    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("D\x11\x06\r\n") {
        println!("Error: Failed to get computer part info ack'd.");
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitAck,
            "Failed to get computer part info ack'd.".to_owned(),
        ));
    }

    Ok(())
}

fn transmit_tasks_loop(f: &mut Box<dyn SerialPort>) -> Result<bool, ExitMsg> {
    if f.write_all(b"H\x30\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatSend,
            "Failed to send heartbeat.".to_owned(),
        ));
    }
    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("H\x31\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatAck,
            "Failed to get heartbeat ack'd.".to_owned(),
        ));
    }

    let m = format!(
        "D\x11\x31{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
        "b0", //"cpuFreq",
        "b1", //"cpuTemp",
        "b2", //"cpuLoad",
        "b3", //"ramUsed",
        "b4", //"gpuTemp",
        "b5", //"gpuCoreClock",
        "b6", //"gpuCoreLoad",
        "b7", //"gpuVramClock",
        "b8", //"gpuVramLoad",
    );
    let m = m.as_bytes();
    if f.write_all(m).is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatSend,
            "Failed to send computer part stats.".to_owned(),
        ));
    }
    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("D\x11\x06\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatAck,
            "Failed to get get computer part stats ack'd.".to_owned(),
        ));
    }

    // TODO: include a clause for closing the connection and returning true.

    Ok(false)
}

pub fn serial_task(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    serial_greet_host(f)?;
    serial_greet_device(f)?;
    serial_link_confirm_host(f)?;
    serial_link_confirm_device(f)?;

    serial_transmit(f)
}
