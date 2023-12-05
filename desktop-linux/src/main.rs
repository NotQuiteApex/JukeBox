// A lightweight program to simulate JukeBox serial communication.

use std::{io::{Write, Read}, ops::Add};

#[derive(PartialEq)]
enum SerialStage {
    ErrorWait,
    GreetHost,
    GreetDevice,
    LinkConfirmHost,
    LinkConfirmDevice,
    TransmitReady,
}

fn print_help(cli: &str) {
    println!("Usage: {} [FILE]", cli);
    println!("Reads and writes to FILE as serial communication.");
    std::process::exit(1)
}

fn get_serial(f: &mut std::fs::File) -> Result<String, ()> {
    let timeout = std::time::SystemTime::now().add(std::time::Duration::from_secs(3));
    let mut buf = Vec::new();

    loop {
        if std::time::SystemTime::now() >= timeout {
            return Err(());
        }

        let res = f.read_to_end(&mut buf);
        if res.is_err() {
            println!("{:?}", res);
            return Err(());
        }

        let len = buf.len();
        if buf.get(len-2).map_or(false, |v| v == &b'\n') && buf.get(len-1).map_or(false, |v| v == &b'\n') {
            // we matched the string, return
            let s = String::from_utf8(buf);
            if s.is_err() {
                return Err(());
            }
            return Ok(s.unwrap());
        }
    }
}

fn transmit_tasks_init(f: &mut std::fs::File) -> bool {
    let m = format!("D\x11\x30{}\x1F{}\x1F{}GB\x1F\r\n", "TestCPU", "TestGPU", "0");
    let m = m.as_bytes();

    if f.write_all(m).is_err() {
        println!("Error: Failed to send computer part info.");
        return true;
    }

    let s = get_serial(f);
    if s.is_err() || s.unwrap() != String::from("D\x11\x06\r\n") {
        println!("Error: Failed to get computer part info ack'd.");
        return true;
    }

    false
}

fn transmit_tasks_loop(f: &mut std::fs::File) -> bool {
    loop {
        if f.write_all(b"H\x30\r\n").is_err() {
            println!("Error: Failed to send heartbeat.");
            return true;
        }
        let s = get_serial(f);
        if s.is_err() || s.unwrap() != String::from("H\x31\r\n") {
            println!("Error: Failed to get heartbeat ack'd.");
            return true;
        }

        let m = format!(
            "D\x11\x31{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F{}\x1F\r\n",
            "cpuFreq",
            "cpuTemp",
            "cpuLoad",
            "ramUsed",
            "gpuTemp",
            "gpuCoreClock",
            "gpuCoreLoad",
            "gpuVramClock",
            "gpuVramLoad",
        );
        let m = m.as_bytes();
        if f.write_all(m).is_err() {
            println!("Error: Failed to send computer part stats.");
            return true;
        }
        let s = get_serial(f);
        if s.is_err() || s.unwrap() != String::from("H\x31\r\n") {
            println!("Error: Failed to get heartbeat ack'd.");
            return true;
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let exe = args.get(0).map_or("jukeboxcli", |v| v.as_str());
    if args.len() != 2 {
        println!("Invalid arguments.");
        print_help(exe);
        return;
    }

    let f = std::fs::File::open(&args[1]);
    if f.is_err() {
        println!("Cannot open serial file.");
        print_help(exe);
    }
    let mut f = f.unwrap();

    let mut stage = SerialStage::GreetHost;

    loop {
        if stage == SerialStage::ErrorWait {
            // We experienced an error of some kind, so we need to wait.
            println!("SerialStage: ErrorWait");
            std::thread::sleep(std::time::Duration::from_secs(1));
            stage = SerialStage::GreetHost;
            // TODO: check if file is gone?
        }

        if stage == SerialStage::GreetHost {
            // Send enquire message to JukeBox
            println!("SerialStage: GreetHost");
            if f.write_all(b"JB\x05\r\n").is_err() {
                println!("Error: Failed to send greeting.");
                stage = SerialStage::ErrorWait;
                continue;
            }
            stage = SerialStage::GreetDevice;
        }

        if stage == SerialStage::GreetDevice {
            // Recieve response, which should contain protocol version.
            println!("SerialStage: GreetHost");
            let s = get_serial(&mut f);
            if s.is_err() || s.unwrap() != String::from("P001\r\n") {
                // response was bad, go to error state.
                println!("Error: Failed to confirm protocol.");
                stage = SerialStage::ErrorWait;
                // TODO: send nack in response to bad protocol
                continue;
            }
            stage = SerialStage::LinkConfirmHost;
        }

        if stage == SerialStage::LinkConfirmHost {
            println!("SerialStage: LinkConfirmHost");
            if f.write_all(b"P\x06\r\n").is_err() {
                println!("Error: Failed to send protocol ack.");
                stage = SerialStage::ErrorWait;
                continue;
            }
            stage = SerialStage::LinkConfirmDevice;
        }

        if stage == SerialStage::LinkConfirmDevice {
            println!("SerialStage: LinkConfirmDevice");
            let s = get_serial(&mut f);
            if s.is_err() || s.unwrap() != String::from("L\x06\r\n") {
                // response was bad, go to error state.
                println!("Error: Failed to confirm link.");
                stage = SerialStage::ErrorWait;
                continue;
            }
            stage = SerialStage::TransmitReady;
            println!("SerialStage: TransmitReady");
        }

        if stage == SerialStage::TransmitReady {
            if transmit_tasks_init(&mut f) || transmit_tasks_loop(&mut f) {
                stage = SerialStage::ErrorWait;
            }
        }
    }
}
