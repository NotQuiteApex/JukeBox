//! Serial processing module

#[allow(unused_imports)]
use defmt::*;

use embedded_hal::timer::{Cancel as _, CountDown as _};
use itertools::Itertools;
use jukebox_util::{
    peripheral::{
        Connection, JBInputs, IDENT_KEY_INPUT, IDENT_KNOB_INPUT, IDENT_PEDAL_INPUT,
        IDENT_UNKNOWN_INPUT,
    },
    protocol::{
        Command, CMD_END, RSP_DISCONNECTED, RSP_END, RSP_INPUT_HEADER, RSP_LINK_DELIMITER,
        RSP_LINK_HEADER, RSP_UNKNOWN,
    },
};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rp_pico::hal::{fugit::ExtU32, timer::CountDown, usb::UsbBus};
use usbd_serial::SerialPort;

use crate::mutex::Mutex;
use crate::peripheral::{inputs_default, inputs_write_report};

const BUFFER_SIZE: usize = 2048;

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
            state: Connection::NotConnected(true),
            keepalive_timer: timer,
        }
    }

    fn check_cmd(&mut self) -> Option<usize> {
        // we measure out a command token by looking for the end-of-command string: "\r\n"
        // if one is not found, we do not have a valid command ready to be read
        // TODO: better match the characters in CMD_END
        for ((_, r), (i, n)) in self.buffer.iter().enumerate().tuple_windows() {
            if *r == CMD_END[0] && *n == CMD_END[1] {
                return Some(i + 1);
            }
        }

        None
    }

    fn send(serial: &mut SerialPort<UsbBus>, rsp: &[u8]) {
        // TODO: its possible for write to drop some characters, if we're not careful.
        // we should probably handle that before we take on larger communications.
        while let Err(_) = serial.write(rsp) {
            let _ = serial.flush();
            cortex_m::asm::nop();
        }

        // match serial.write(rsp) {
        //     Err(_) => core::todo!(),
        //     Ok(_) => {}
        // };
    }

    fn send_full_response(serial: &mut SerialPort<UsbBus>, rsp: &[u8]) {
        Self::send(serial, rsp);
        Self::send_end_response(serial);
    }

    fn send_end_response(serial: &mut SerialPort<UsbBus>) {
        Self::send(serial, RSP_END);
    }

    fn decode_cmd(&mut self, size: usize) -> Command {
        if size > 4 {
            return Command::Unknown;
        }

        let w1 = self.buffer.get(0).unwrap_or(&b'\0');
        let w2 = self.buffer.get(1).unwrap_or(&b'\0');
        let w3 = self.buffer.get(2).unwrap_or(&b'\0');

        debug!("cmd: {} {} {} (size:{})", w1, w2, w3, size);

        let cmd = Command::decode(*w1);

        if cmd != Command::Unknown && !(*w2 == CMD_END[0] && *w3 == CMD_END[1]) {
            for _ in 0..size {
                self.buffer.dequeue();
            }

            return Command::Unknown;
        }

        for _ in 0..size {
            self.buffer.dequeue();
        }

        cmd
    }

    #[allow(dead_code)]
    pub fn get_connection_status(&self) -> Connection {
        self.state.clone()
    }

    fn start_update(&mut self, serial: &mut SerialPort<UsbBus>, update_trigger: &Mutex<2, bool>) {
        info!("Command Update");
        Self::send_full_response(serial, &[RSP_DISCONNECTED]);
        self.state = Connection::NotConnected(true);
        update_trigger.with_mut_lock(|u| *u = true);
    }

    pub fn update(
        &mut self,
        serial: &mut SerialPort<UsbBus>,
        firmware_version: &str,
        device_uid: &str,
        peripheral_inputs: &Mutex<1, JBInputs>,
        update_trigger: &Mutex<2, bool>,
    ) {
        if self.state == Connection::Connected && self.keepalive_timer.wait().is_ok() {
            warn!("Keepalive triggered, disconnecting.");
            self.state = Connection::NotConnected(false);
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
            Self::send_full_response(serial, &[RSP_UNKNOWN]);
            false
        };
        let valid = match self.state {
            Connection::NotConnected(_) => match decode {
                Command::Update => {
                    self.start_update(serial, update_trigger);
                    true
                }
                Command::Greeting => {
                    Self::send(serial, &[RSP_LINK_HEADER, RSP_LINK_DELIMITER]);
                    let dtype = if cfg!(feature = "keypad") {
                        IDENT_KEY_INPUT
                    } else if cfg!(feature = "knobpad") {
                        IDENT_KNOB_INPUT
                    } else if cfg!(feature = "pedalpad") {
                        IDENT_PEDAL_INPUT
                    } else {
                        IDENT_UNKNOWN_INPUT
                    };
                    Self::send(serial, &[dtype]);
                    Self::send(serial, &[RSP_LINK_DELIMITER]);
                    Self::send(serial, firmware_version.as_bytes());
                    Self::send(serial, &[RSP_LINK_DELIMITER]);
                    Self::send(serial, device_uid.as_bytes());
                    Self::send(serial, &[RSP_LINK_DELIMITER]);
                    Self::send_end_response(serial);

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
                Command::GetInputKeys => {
                    // copy peripherals and inputs out
                    let inputs = {
                        let mut inputs = inputs_default(); // JBInputs::default();
                        peripheral_inputs.with_lock(|i| {
                            inputs = *i;
                        });
                        inputs
                    };

                    // write all the inputs out
                    Self::send(serial, &[RSP_INPUT_HEADER]);
                    inputs_write_report(inputs, serial);
                    Self::send_end_response(serial);

                    true
                }
                Command::NegativeAck => {
                    // we sent something in error, better bail
                    self.state = Connection::NotConnected(false);
                    info!("Serial NegativeAck'd");
                    false
                }
                Command::Disconnect => {
                    Self::send_full_response(serial, &[RSP_DISCONNECTED]);
                    self.state = Connection::NotConnected(true);
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
