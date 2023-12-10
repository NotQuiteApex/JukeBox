// Serial communication

use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use std::ops::Add;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use sysinfo::{CpuExt, System, SystemExt};

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

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Send enquire message to JukeBox
    log::debug!("SerialStage: GreetHost");

    if f.write_all(b"JB\x05\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageGreetHost,
            "Failed to send host greeting.".to_owned(),
        ));
    }
    f.flush();

    Ok(())
}

fn greet_device(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Recieve response, which should contain protocol version.
    log::debug!("SerialStage: GreetDevice");

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
    log::debug!("SerialStage: LinkConfirmHost");

    if f.write_all(b"P\x06\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageLinkConfirmHost,
            "Failed to send protocol ack.".to_owned(),
        ));
    }
    f.flush();

    Ok(())
}

fn link_confirm_device(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    log::debug!("SerialStage: LinkConfirmDevice");

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

fn transmit_tasks_init(f: &mut Box<dyn SerialPort>, sys: &System) -> Result<(), ExitMsg> {
    let cpu_name = sys.global_cpu_info().brand().trim();
    let gpu_name = "TestGPU";
    let memory = format!("{:.1}", (sys.total_memory() as f64) / ((1 << 30) as f64));

    let m = format!(
        "D\x11\x30{}\x1F{}\x1F{:.1}GiB\x1F\r\n",
        cpu_name, gpu_name, memory,
    );
    let m = m.as_bytes();
    if m.len() > 128 {
        log::warn!("TInit string longer than 64 bytes!");
        log::warn!("TInit CPU Brand: '{}'.", cpu_name);
        log::warn!("TInit GPU Brand: '{}'.", gpu_name);
        log::warn!("TInit Memory: '{}'.", memory);
        log::warn!("TInit string len: {} bytes.", m.len());
        // log::warn!("TInit string: {:?}", m);
    }

    f.write_all(m).map_err(|why| {
        ExitMsg::new(
            ExitCode::SerialTransmitComputerPartInitSend,
            format!("Failed to send computer part info, reason '{}'.", why),
        )
    })?;
    f.flush();

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

fn transmit_tasks_loop(f: &mut Box<dyn SerialPort>, sys: &System) -> Result<bool, ExitMsg> {
    if f.write_all(b"H\x30\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatSend,
            "Failed to send heartbeat.".to_owned(),
        ));
    }
    f.flush();
    let s = get_serial_string(f);
    if s.is_err() || s.unwrap() != String::from("H\x31\r\n") {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitHeartbeatAck,
            "Failed to get heartbeat ack'd.".to_owned(),
        ));
    }

    // let cpu_freq = sys.global_cpu_info().frequency();
    // let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    // let mem_used = sys.used_memory();

    let m = format!(
        "D\x11\x31{:.2}\x1F{}\x1F{:.1}\x1F{:.1}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
        (sys.global_cpu_info().frequency() as f64) / (1000 as f64), // "b0", //"cpuFreq",
        "b1",                                                       //"cpuTemp",
        sys.global_cpu_info().cpu_usage(),                          // "b2", //"cpuLoad",
        (sys.used_memory() as f64) / ((1 << 30) as f64),            // "b3", //"ramUsed",
        "b4",                                                       //"gpuTemp",
        "b5",                                                       //"gpuCoreClock",
        "b6",                                                       //"gpuCoreLoad",
        "b7",                                                       //"gpuVramClock",
        "b8",                                                       //"gpuVramLoad",
    );
    let m = m.as_bytes();
    if f.write_all(m).is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialTransmitComputerPartStatSend,
            "Failed to send computer part stats.".to_owned(),
        ));
    }
    f.flush();
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
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.refresh_memory();

    greet_host(f)?;
    greet_device(f)?;
    link_confirm_host(f)?;
    link_confirm_device(f)?;

    transmit_tasks_init(f, &sys)?;

    let mut timer = SystemTime::now();

    loop {
        if SystemTime::now() < timer {
            continue;
        }

        if transmit_tasks_loop(f, &sys)? {
            break;
        }

        sys.refresh_cpu();
        sys.refresh_memory();
        // sleep(Duration::from_millis(900));
        timer = SystemTime::now().add(Duration::from_millis(500));
    }

    Ok(())
}
