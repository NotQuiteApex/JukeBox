//! Serial processing module

use rp_pico::hal::usb::UsbBus;
use usbd_serial::SerialPort;

pub struct SerialMod {}

impl SerialMod {
    pub fn new() -> Self {
        SerialMod {}
    }

    pub fn update(&mut self, serial: &mut SerialPort<UsbBus>) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_) => {}
            Ok(0) => {}
            Ok(count) => {
                buf.iter_mut()
                    .take(count)
                    .for_each(|b| b.make_ascii_uppercase());
                let mut wr_ptr = &buf[..count];
                while !wr_ptr.is_empty() {
                    match serial.write(wr_ptr) {
                        Ok(len) => wr_ptr = &wr_ptr[len..],
                        Err(_) => break,
                    }
                }
            }
        }
    }
}
