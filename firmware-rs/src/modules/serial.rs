//! Serial processing module

use defmt::info;
use embedded_hal::timer::CountDown as _;
use itertools::Itertools;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::{fugit::ExtU32, timer::CountDown, usb::UsbBus};
use usbd_serial::SerialPort;

const BUFFER_SIZE: usize = 2048;

const PERIPHERAL_ID_KEYBOARD: u8 = 0b1000_0000;
const PERIPHERAL_ID_KNOBS_1: u8 = 0b1000_0010;
const PERIPHERAL_ID_KNOBS_2: u8 = 0b1000_0011;
const PERIPHERAL_ID_PEDAL_1: u8 = 0b1000_0101;
const PERIPHERAL_ID_PEDAL_2: u8 = 0b1000_0110;
const PERIPHERAL_ID_PEDAL_3: u8 = 0b1000_0111;

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

    fn start_update(&mut self, serial: &mut SerialPort<UsbBus>) {
        info!("Command Update");
        // we don't care what state we're in for an update,
        // because we need to be able to update whenever.
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
                    todo!();
                    let _ = serial.write(b"I");
                    // TODO: get all current key strokes from a mutex?
                    // TODO: send these conditionally
                    let _ = serial.write(&[PERIPHERAL_ID_KEYBOARD, 0, 0]);
                    let _ = serial.write(&[PERIPHERAL_ID_KNOBS_1, 0]);
                    let _ = serial.write(&[PERIPHERAL_ID_KNOBS_2, 0]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_1, 0]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_2, 0]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_3, 0]);
                    Self::send_response(serial, b"\r\n");
                    // we'll need to pack bits for efficient transfer
                    // write I
                    // write peripheral id word (0b0 - keyboard, 0b1XX - pedal, 0b1X - knob), then write peripheral word
                    // keyboard word - 16 bit word, each bit for a key (0 up, 1 pressed)
                    // pedal pad word - 8 bit word per pedal, last 3 bits are keys
                    // knob word - last bits are 2 sets of 3 bits;
                    // -- first bit is key, second two are rotation (00 - none, 10 - CCW, 01 - CW)
                    // write \r\n
                    true
                }
                Command::GetPeripherals => {
                    info!("Command GetPeripherals");
                    let _ = serial.write(b"A");
                    // TODO: send these conditionally
                    let _ = serial.write(&[PERIPHERAL_ID_KEYBOARD]);
                    let _ = serial.write(&[PERIPHERAL_ID_KNOBS_1]);
                    let _ = serial.write(&[PERIPHERAL_ID_KNOBS_2]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_1]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_2]);
                    let _ = serial.write(&[PERIPHERAL_ID_PEDAL_3]);
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
