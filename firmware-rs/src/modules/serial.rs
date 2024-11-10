//! Serial processing module

use defmt::info;
use embedded_hal::timer::CountDown as _;
use itertools::Itertools;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::{fugit::ExtU32, timer::CountDown, usb::UsbBus};
use usbd_serial::SerialPort;

const BUFFER_SIZE: usize = 1024;

const RSP_HEARTBEAT: &[u8] = b"H\r\n";
const RSP_UNKNOWN: &[u8] = b"?\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n";

#[derive(PartialEq, Clone)]
pub enum SerialState {
    NotConnected,
    Connected,
}

enum Command {
    Greeting,
    Heartbeat,

    GetInputKeys,
    GetPeripherals,

    Update,
    Disconnect,
    Test,
    Unknown,
}

const KEEPALIVE: u32 = 100;

pub struct SerialMod<'timer> {
    buffer: ConstGenericRingBuffer<u8, BUFFER_SIZE>,
    state: SerialState,
    keepalive_timer: CountDown<'timer>,
}

impl<'timer> SerialMod<'timer> {
    pub fn new(mut timer: CountDown<'timer>) -> Self {
        timer.start(KEEPALIVE.millis());

        SerialMod {
            buffer: ConstGenericRingBuffer::new(),
            state: SerialState::NotConnected,
            keepalive_timer: timer,
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
            Ok(_) => match serial.flush() {
                Err(_) => {}
                Ok(()) => {}
            },
        };
    }

    fn decode_cmd(&mut self, size: usize) -> Command {
        let res = match self.buffer.get(0).unwrap() {
            b'\x05' => Command::Greeting,
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
        self.state.clone()
    }

    fn start_update(&mut self, serial: &mut SerialPort<UsbBus>) {
        info!("Command Update");
        // We don't care what state we're in for an update, because we need to be able to update whenever.
        Self::send_response(serial, RSP_DISCONNECTED);
        self.state = SerialState::NotConnected;
        // reset_to_usb_boot(0, 0);
        todo!();
        // TODO: schedule a reset_to_usb_boot call,
        // make sure to cleanly stop all other modules and clear screen and rgb.
    }

    pub fn update(
        &mut self,
        serial: &mut SerialPort<UsbBus>,
        firmware_version: &str,
        device_uid: &str,
    ) {
        if self.state == SerialState::Connected && self.keepalive_timer.wait().is_ok() {
            info!("Keepalive triggered.");
            self.state = SerialState::NotConnected;
        }

        let mut buf = [0u8; 128];
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
        let mut unknown = || {
            Self::send_response(serial, RSP_UNKNOWN);
            false
        };
        let valid = match self.state {
            SerialState::NotConnected => match decode {
                Command::Update => {
                    self.start_update(serial);
                    true
                }
                Command::Greeting => {
                    let _ = serial.write(b"L,");
                    let _ = serial.write(firmware_version.as_bytes());
                    let _ = serial.write(b",");
                    let _ = serial.write(device_uid.as_bytes());
                    Self::send_response(serial, b",\r\n");

                    self.state = SerialState::Connected;
                    info!("Serial Connected");
                    true
                }
                _ => unknown(),
            },
            SerialState::Connected => match decode {
                Command::Update => {
                    self.start_update(serial);
                    true
                }
                Command::Test => {
                    info!("Command Test");
                    true
                }
                Command::Heartbeat => {
                    Self::send_response(serial, RSP_HEARTBEAT);
                    true
                }
                Command::GetInputKeys => {
                    info!("Command GetInputKeys");
                    // TODO: get all current key strokes from a mutex?
                    todo!();
                    // true
                }
                Command::GetPeripherals => {
                    info!("Command GetPeripherals");
                    // A - Peripheral Response Indicator
                    // K - Keyboard Peripheral
                    // N - Knobs 1 Peripheral
                    // O - Knobs 2 Peripheral
                    // P - Pedal 1 Peripheral
                    // E - Pedal 2 Peripheral
                    // D - Pedal 3 Peripheral
                    Self::send_response(serial, b"AK\r\n");
                    // TODO: get all peripherals from a quick bus scan in other thread
                    true
                }
                Command::Disconnect => {
                    Self::send_response(serial, RSP_DISCONNECTED);
                    self.state = SerialState::NotConnected;
                    info!("Serial Disconnected");
                    true
                }
                _ => unknown(),
            },
        };

        if valid {
            self.keepalive_timer.start(KEEPALIVE.millis()); // restart keepalive timer with valid command
        }
    }
}
