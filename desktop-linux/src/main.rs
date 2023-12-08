// A lightweight program to simulate JukeBox serial communication.

use std::ops::Add;
use std::time::{Duration, SystemTime};
use serialport::SerialPort;

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

fn get_serial(f: &mut Box<dyn SerialPort>) -> Result<String, ()> {
	let timeout = SystemTime::now().add(Duration::from_secs(3));
	let mut buf = Vec::new();

	loop {
		if SystemTime::now() >= timeout {
			println!("Error: Serial read timeout.");
			return Err(());
		}
		
		let mut b: [u8; 1] = [0; 1];
		let res = f.read(&mut b);
		if res.is_err() {
			continue;
			// println!("Failed to read byte from serial.");
			// println!("{:?}", res);
			// return Err(());
		}
		buf.push(b[0]);

		let len = buf.len();
		println!("{:?}", buf);
		if len >= 2 && buf.get(len-2).map_or(false, |v| v == &b'\r') && buf.get(len-1).map_or(false, |v| v == &b'\n') {
			// we matched the string, return
			let s = String::from_utf8(buf);
			if s.is_err() {
				return Err(());
			}
			return Ok(s.unwrap());
		}
	}
}

fn transmit_tasks_init(f: &mut Box<dyn SerialPort>) -> bool {
	let m = format!("D\x11\x30{}\x1F{}\x1F{}GB\x1F\r\n",
		"a0", //"TestCPU",
		"a1", //"TestGPU",
		"a2", //"0",
	);
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

fn transmit_tasks_loop(f: &mut Box<dyn SerialPort>) -> bool {
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
			println!("Error: Failed to send computer part stats.");
			return true;
		}
		let s = get_serial(f);
		if s.is_err() || s.unwrap() != String::from("D\x11\x06\r\n") {
			println!("Error: Failed to get get computer part stats ack'd.");
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

	let md = std::fs::metadata(&args[1]);
	if md.is_err() {
		println!("Failed to read FILE metadata.");
		print_help(exe);
		return;
	}
	let md = md.unwrap();
	if md.is_dir() {
		println!("FILE cannot be a directory.");
		print_help(exe);
		return;
	}

	let f = serialport::new(&args[1], 115200)
		// .timeout(Duration::from_millis(10))
		.open();
	if f.is_err() {
		println!("Cannot open serial file.");
		print_help(exe);
		return;
	}
	let mut f = f.unwrap();

	let mut stage = SerialStage::GreetHost;

	loop {
		if stage == SerialStage::ErrorWait {
			// We experienced an error of some kind, so we need to wait.
			println!("SerialStage: ErrorWait");
			std::thread::sleep(Duration::from_secs(1));
			stage = SerialStage::GreetHost;
			// TODO: check if file is gone?
		}

		if stage == SerialStage::GreetHost {
			// Send enquire message to JukeBox
			println!("SerialStage: GreetHost");
			let e = f.write_all(b"JB\x05\r\n");
			if e.is_err() {
				println!("Error: Failed to send greeting.");
				println!("{:?}", e);
				stage = SerialStage::ErrorWait;
				continue;
			}
			stage = SerialStage::GreetDevice;
		}

		if stage == SerialStage::GreetDevice {
			// Recieve response, which should contain protocol version.
			println!("SerialStage: GreetDevice");

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
