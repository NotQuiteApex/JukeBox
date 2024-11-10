//! Serial processing module

use defmt::info;
use itertools::Itertools;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::usb::UsbBus;
use usbd_serial::SerialPort;

const BUFFER_SIZE: usize = 1024;

const RSP_PROTOCOL: &[u8] = b"P001\r\n";
const RSP_LINK_ESTABLISHED: &[u8] = b"L\r\n";
const RSP_HEARTBEAT: &[u8] = b"H\r\n";
const RSP_UNKNOWN: &[u8] = b"?\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n";

#[derive(PartialEq)]
pub enum SerialState {
    NotConnected,
    Greeted,
    Connected,
}

enum Command {
    Unknown,

    Greeting,
    ProtocolAccept,
    Heartbeat,

    GetInputKeys,
    GetPeripherals,

    Update,
    Disconnect,
    Test,
}

pub struct SerialMod {
    buffer: ConstGenericRingBuffer<u8, BUFFER_SIZE>,
    state: SerialState,
}

impl SerialMod {
    pub fn new() -> Self {
        SerialMod {
            buffer: ConstGenericRingBuffer::new(),
            state: SerialState::NotConnected,
        }
    }

    fn check_cmd(&mut self) -> Option<usize> {
        for ((_, r), (i, n)) in self.buffer.iter().enumerate().tuple_windows() {
            if *r == b'\r' && *n == b'\n' {
                return Some(i + 1);
            }
        }

        None
    }

    fn send_response(serial: &mut SerialPort<UsbBus>, rsp: &[u8]) {
        match serial.write(rsp) {
            Err(_) => todo!(),
            Ok(s) => {
                if rsp.len() != s {
                    todo!()
                }

                match serial.flush() {
                    Err(_) => todo!(),
                    Ok(()) => {}
                }
            }
        };
    }

    fn decode_cmd(&mut self, size: usize) -> Command {
        let res = match self.buffer.get(0).unwrap() {
            b'\x05' => Command::Greeting,
            b'P' => Command::ProtocolAccept,
            b'H' => Command::Heartbeat,
            b'U' => {
                let p2 = self.buffer.get(1);
                if p2.is_none() {
                    return Command::Unknown;
                }

                match p2.unwrap() {
                    b'\x30' => Command::GetInputKeys,
                    b'\x31' => Command::GetPeripherals,

                    b'\x37' => Command::Test,
                    b'\x38' => Command::Update,
                    b'\x39' => Command::Disconnect,

                    _ => Command::Unknown,
                }
            }
            _ => Command::Unknown,
        };

        // consume command buffer data
        for _ in 0..size {
            self.buffer.dequeue();
        }

        res
    }

    pub fn _get_connection_status(&self) -> SerialState {
        match self.state {
            SerialState::Connected => SerialState::Connected,
            _ => SerialState::NotConnected,
        }
    }

    pub fn update(&mut self, serial: &mut SerialPort<UsbBus>) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_) => {}
            Ok(s) => {
                // copy read data to internal buffer
                for b in 0..s {
                    self.buffer.push(buf[b]);
                }
            }
        }

        // load and decode command if available
        let size = self.check_cmd();
        if size.is_none() {
            return;
        }
        let decode = self.decode_cmd(size.unwrap());

        // process command
        let _res = match decode {
            Command::Greeting => {
                if self.state == SerialState::NotConnected {
                    Self::send_response(serial, RSP_PROTOCOL);
                    self.state = SerialState::Greeted;
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }
            Command::ProtocolAccept => {
                if self.state == SerialState::Greeted {
                    Self::send_response(serial, RSP_LINK_ESTABLISHED);
                    self.state = SerialState::Connected;
                    info!("Serial Connected");
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }
            Command::Heartbeat => {
                if self.state == SerialState::Connected {
                    Self::send_response(serial, RSP_HEARTBEAT)
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }

            Command::GetInputKeys => {
                if self.state == SerialState::Connected {
                    info!("Command GetInputKeys");
                    todo!() // TODO: get all current key strokes from a mutex?
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }
            Command::GetPeripherals => {
                if self.state == SerialState::Connected {
                    info!("Command GetPeripherals");
                    todo!() // TODO: get all peripherals from a quick bus scan in other thread
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }

            Command::Update => {
                info!("Command Update");
                // We don't care what state we're in for an update, because we need to be able to update whenever.
                Self::send_response(serial, RSP_DISCONNECTED);
                self.state = SerialState::NotConnected;
                // reset_to_usb_boot(0, 0);
                todo!();
                // TODO: schedule a reset_to_usb_boot call,
                // make sure to cleanly stop all other modules and clear screen and rgb.
            }
            Command::Disconnect => {
                if self.state == SerialState::Connected {
                    Self::send_response(serial, RSP_DISCONNECTED);
                    self.state = SerialState::NotConnected;
                    info!("Serial Disconnected");
                } else {
                    Self::send_response(serial, RSP_UNKNOWN);
                }
            }
            Command::Test => info!("Command Test"),

            Command::Unknown => {
                // We don't know what the host wanted, so we'll communicate as such.
                Self::send_response(serial, RSP_UNKNOWN);
            }
        };
    }
}
