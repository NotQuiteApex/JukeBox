//! Serial processing module

use itertools::Itertools;
use rp_pico::hal::usb::UsbBus;
use usbd_serial::SerialPort;

const BUFFER_SIZE: usize = 128;

pub struct SerialMod {
    buffer: [u8; BUFFER_SIZE],
    buffer_ptr: usize,
}

impl SerialMod {
    pub fn new() -> Self {
        SerialMod {
            buffer: [0u8; BUFFER_SIZE],
            buffer_ptr: 0,
        }
    }

    fn get_cmd(&mut self) -> Option<usize> {
        for ((_, r), (i, n)) in self.buffer.iter().enumerate().tuple_windows() {
            if *r == b'\r' && *n == b'\n' {
                return Some(i);
            }
        }

        None
    }

    fn process_cmd(&mut self, cmd: &[u8]) {
        match cmd[0] {
            _ => panic!("Unrecognized command!"),
        }
    }

    pub fn update(&mut self, serial: &mut SerialPort<UsbBus>) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_) => {}
            Ok(0) => {}
            Ok(_) => {
                // copy read data to internal buffer
                for b in buf {
                    assert!(self.buffer_ptr < BUFFER_SIZE);
                    self.buffer[self.buffer_ptr] = b;
                    self.buffer_ptr += 1;
                }
            }
        }

        let cmd = self.get_cmd();
        if cmd.is_none() {
            return;
        }
        let size = cmd.unwrap();
        let cmd = &self.buffer[..=size];

        // TODO: process cmd

        // TODO: fix self.buffer (and check that this works)
        self.buffer.rotate_left(size);
    }
}
