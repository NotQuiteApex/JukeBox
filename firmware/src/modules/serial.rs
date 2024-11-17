//! Serial processing module

use defmt::info;
use defmt::warn;
use embedded_hal::timer::Cancel as _;
use embedded_hal::timer::CountDown as _;
use itertools::Itertools;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::{fugit::ExtU32, timer::CountDown, usb::UsbBus};
use usbd_serial::SerialPort;

use crate::mutex::Mutex;
use crate::peripheral::{Connection, JBPeripheralInputs, JBPeripherals};

const BUFFER_SIZE: usize = 2048;

const RSP_UNKNOWN: &[u8] = b"?\r\n\r\n";
const RSP_DISCONNECTED: &[u8] = b"\x04\x04\r\n\r\n";

#[derive(defmt::Format)]
enum Command {
    Greeting,

    GetInputKeys,
    GetPeripherals,

    Update,
    Disconnect,
    NegativeAck,
    Test,
    Unknown,
}

const KEEPALIVE: u32 = 250;

pub struct SerialMod<'timer> {
    buffer: ConstGenericRingBuffer<u8, BUFFER_SIZE>,
    state: Connection,
    keepalive_timer: CountDown<'timer>,
}

impl<'timer> SerialMod<'timer> {
    pub fn new(mut timer: CountDown<'timer>) -> Self {
        timer.start(KEEPALIVE.millis());

        SerialMod {
            buffer: ConstGenericRingBuffer::new(),
            state: Connection::NotConnected,
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
        match serial.write(rsp) {
            Err(_) => todo!(),
            Ok(_) => {}
        };
    }

    fn decode_cmd(&mut self, size: usize) -> Command {
        // we parse the command strings into tokens here
        // null characters are placeholders for errors or missing characters
        // (there are exceptions to this rule, but only for the desktop parser)
        // the character string "\r\n\r\n" indicates the end of a response
        // its unlikely we'd run into the end of the buffer due to check_cmd()
        // but we use unwrap_or() anyway for safety

        let mut depth = 1;
        let res = match self.buffer.get(0).unwrap_or(&b'\0') {
            b'\x05' => Command::Greeting,
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
            b'\x15' => Command::NegativeAck,
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

    pub fn get_connection_status(&self) -> Connection {
        self.state.clone()
    }

    fn start_update(&mut self, serial: &mut SerialPort<UsbBus>, update_trigger: &Mutex<2, bool>) {
        info!("Command Update");
        Self::send_response(serial, RSP_DISCONNECTED);
        self.state = Connection::NotConnected;
        update_trigger.with_mut_lock(|u| {
            *u = true;
        });
    }

    pub fn update(
        &mut self,
        serial: &mut SerialPort<UsbBus>,
        firmware_version: &str,
        device_uid: &str,
        connected_peripherals: &Mutex<0, JBPeripherals>,
        peripheral_inputs: &Mutex<1, JBPeripheralInputs>,
        update_trigger: &Mutex<2, bool>,
    ) {
        if self.state == Connection::Connected && self.keepalive_timer.wait().is_ok() {
            warn!("Keepalive triggered, disconnecting.");
            self.state = Connection::NotConnected;
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
            Connection::NotConnected => match decode {
                Command::Update => {
                    self.start_update(serial, update_trigger);
                    true
                }
                Command::Greeting => {
                    Self::send_response(serial, b"L,");
                    Self::send_response(serial, firmware_version.as_bytes());
                    Self::send_response(serial, b",");
                    Self::send_response(serial, device_uid.as_bytes());
                    Self::send_response(serial, b",\r\n\r\n");

                    self.state = Connection::Connected;
                    info!("Serial Connected");
                    true
                }
                _ => unknown(),
            },
            Connection::Connected => match decode {
                Command::Update => {
                    self.start_update(serial, update_trigger);
                    true
                }
                Command::Test => {
                    info!("Command Test");
                    true
                }
                Command::GetInputKeys => {
                    // info!("Command GetInputKeys");

                    // copy peripherals and inputs out
                    let mut peripherals = JBPeripherals::default();
                    let mut inputs = JBPeripheralInputs::default();
                    connected_peripherals.with_lock(|c| {
                        peripherals = *c;
                    });
                    peripheral_inputs.with_lock(|i| {
                        inputs = *i;
                    });
                    let peripherals = peripherals;
                    let inputs = inputs;

                    // write all the inputs out
                    Self::send_response(serial, b"I");
                    inputs.write_report(peripherals, serial);
                    Self::send_response(serial, b"\r\n\r\n");

                    true
                }
                Command::GetPeripherals => {
                    info!("Command GetPeripherals");

                    // copy peripherals out
                    let mut peripherals = JBPeripherals::default();
                    connected_peripherals.with_lock(|c| {
                        peripherals = *c;
                    });
                    let peripherals = peripherals;

                    Self::send_response(serial, b"A");
                    peripherals.write_report(serial);
                    Self::send_response(serial, b"\r\n\r\n");
                    true
                }
                Command::NegativeAck => {
                    // we sent something in error, better bail
                    self.state = Connection::NotConnected;
                    info!("Serial NegativeAck'd");
                    false
                }
                Command::Disconnect => {
                    Self::send_response(serial, RSP_DISCONNECTED);
                    self.state = Connection::NotConnected;
                    info!("Serial Disconnected");
                    true
                }
                _ => unknown(),
            },
        };

        if valid {
            // info!("restarting keepalive");
            let _ = self.keepalive_timer.cancel();
            self.keepalive_timer.start(KEEPALIVE.millis()); // restart keepalive timer with valid command
        }
    }
}
