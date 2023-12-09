// Serial communication

use crate::util::{ExitCode, ExitMsg};

use serialport::SerialPort;
use sysinfo::{CpuExt, System, SystemExt};
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

fn greet_host(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    // Send enquire message to JukeBox
    log::debug!("SerialStage: GreetHost");

    if f.write_all(b"JB\x05\r\n").is_err() {
        return Err(ExitMsg::new(
            ExitCode::SerialStageGreetHost,
            "Failed to send host greeting.".to_owned(),
        ));
    }

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
    log::debug!("CPU Brand: '{}'.", sys.global_cpu_info().brand());
    log::debug!("Memory: {} Bytes.", sys.total_memory());

    let cpu = sys.global_cpu_info().brand().replace("12th Gen Intel(R) Core(TM)", "Intel Core");
    let m = format!(
        "D\x11\x30{}\x1F{}\x1F{:.1}GiB\x1F\r\n",
        cpu,
        "TEST_GPU_DO_NOT_STEAL", //"TestGPU",
        (sys.total_memory() as f64) / ((1 << 30) as f64)
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

fn transmit_tasks_loop(f: &mut Box<dyn SerialPort>, sys: &System) -> Result<bool, ExitMsg> {
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

    let cpu_freq = sys.global_cpu_info().frequency();
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    let mem_used = sys.used_memory();

    let m = format!(
        "D\x11\x31{:.2}\x1F{}\x1F{:.1}\x1F{:.1}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
        (sys.global_cpu_info().frequency() as f64) / (1000 as f64), // "b0", //"cpuFreq",
        "b1", //"cpuTemp",
        sys.global_cpu_info().cpu_usage(), // "b2", //"cpuLoad",
        (sys.used_memory() as f64) / ((1 << 30) as f64), // "b3", //"ramUsed",
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

    sleep(Duration::from_millis(900));

    Ok(false)
}

pub fn serial_task(f: &mut Box<dyn SerialPort>) -> Result<(), ExitMsg> {
    greet_host(f)?;
    greet_device(f)?;
    link_confirm_host(f)?;
    link_confirm_device(f)?;

    let mut sys = System::new_all();
    sys.refresh_cpu();
    sys.refresh_memory();

    transmit_tasks_init(f, &sys)?;

    loop {
        sys.refresh_cpu();
        sys.refresh_memory();

        let exit = transmit_tasks_loop(f, &sys)?;
        if exit {
            break;
        }
    }

    Ok(())
}
