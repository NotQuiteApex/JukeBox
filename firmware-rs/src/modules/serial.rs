//! Serial processing module

use defmt::info;
use embedded_hal::timer::CountDown as _;
use itertools::Itertools;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::{fugit::ExtU32, timer::CountDown, usb::UsbBus};
use usbd_serial::SerialPort;

use crate::mutex::Mutex;
use crate::peripheral::{ConnectedPeripherals, PeripheralsInputs};

const BUFFER_SIZE: usize = 2048;

const RSP_HEARTBEAT: &[u8] = b"H\r\n";
const RSP_UNKNOWN: &[u8] = b"?\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n";

#[derive(PartialEq, Clone)]
pub enum SerialState {
    NotConnected,
    Connected,
}

#[derive(defmt::Format)]
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
        // we measure out a command token by looking for the end-of-command string: "\r\n"
        // if one is not found, we do not have a valid command ready to be read
        for ((_, r), (i, n)) in self.buffer.iter().enumerate().tuple_windows() {
            if *r == b'\r' && *n == b'\n' {
                return Some(i + 1);
            }
        }

        None
    }

    fn send_response(serial: &mut SerialPort<UsbBus>, rsp: &[u8]) {
        // TODO: its possible for write to drop some characters, if we're not careful.
        // we should probably handle that before we take on larger communications.
        let _ = match serial.write(rsp) {
            Err(_) => todo!(),
            Ok(_) => serial.flush(),
        };
    }

    fn decode_cmd(&mut self, size: usize) -> Command {
        // we parse the command strings into tokens here
        // null characters are placeholders for errors or missing characters
        // (there are exceptions to this rule, but only for the desktop parser)
        // the character string "\r\n" indicates the end of a command
        // its unlikely we'd run into the end of the buffer due to check_cmd()
        // but we use unwrap_or() anyway for safety

        let mut depth = 1;
        let res = match self.buffer.get(0).unwrap_or(&b'\0') {
            b'\x05' => Command::Greeting,
            b'H' => Command::Heartbeat,
            b'U' => {
                let p2 = self.buffer.get(1).unwrap_or(&b'\0');
                depth = 2;

                match p2 {
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

        // we must check the command size matches what we expect the parsed command to be
        // we already know the buffer will end with "\r\n" thanks to check_cmd(),
        // so that's what the +2 is for
        if size != depth + 2 {
            return Command::Unknown;
        }

        res
    }

    pub fn _get_connection_status(&self) -> SerialState {
        self.state.clone()
    }

    fn start_update(
        &mut self,
        serial: &mut SerialPort<UsbBus>,
        update_trigger_mutex: &mut Mutex<2, bool>,
    ) {
        info!("Command Update");
        Self::send_response(serial, RSP_DISCONNECTED);
        self.state = SerialState::NotConnected;
        update_trigger_mutex.with_mut_lock(|u| {
            *u = true;
        });
    }

    pub fn update(
        &mut self,
        serial: &mut SerialPort<UsbBus>,
        firmware_version: &str,
        device_uid: &str,
        connected_peripherals_mutex: &Mutex<0, ConnectedPeripherals>,
        peripheral_inputs_mutex: &Mutex<1, PeripheralsInputs>,
        update_trigger_mutex: &mut Mutex<2, bool>,
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
                    self.start_update(serial, update_trigger_mutex);
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
                    self.start_update(serial, update_trigger_mutex);
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

                    // copy peripherals and inputs out
                    let mut peripherals = ConnectedPeripherals::default();
                    let mut inputs = PeripheralsInputs::default();
                    connected_peripherals_mutex.with_lock(|c| {
                        peripherals = *c;
                    });
                    peripheral_inputs_mutex.with_lock(|i| {
                        inputs = *i;
                    });
                    let peripherals = peripherals;
                    let inputs = inputs;

                    // write all the inputs out
                    let _ = serial.write(b"I");
                    inputs.write_report(peripherals, serial);
                    Self::send_response(serial, b"\r\n");

                    true
                }
                Command::GetPeripherals => {
                    info!("Command GetPeripherals");

                    // copy peripherals out
                    let mut peripherals = ConnectedPeripherals::default();
                    connected_peripherals_mutex.with_lock(|c| {
                        peripherals = *c;
                    });
                    let peripherals = peripherals;

                    let _ = serial.write(b"A");
                    peripherals.write_report(serial);
                    Self::send_response(serial, b"\r\n");
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
