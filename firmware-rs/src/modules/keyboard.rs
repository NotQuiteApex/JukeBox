//! Keyboard processing module

use rp_pico::hal::usb::UsbBus;
use usbd_hid::{
    descriptor::{KeyboardReport, KeyboardUsage},
    hid_class::HIDClass,
};

pub struct KeyboardMod {}

impl KeyboardMod {
    pub fn new() -> Self {
        KeyboardMod {}
    }

    pub fn update(&mut self, hid: &mut HIDClass<UsbBus>) {
        let rep = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [KeyboardUsage::KeyboardF13 as u8, 0, 0, 0, 0, 0],
        };
        let res = hid.push_input(&rep);

        // TODO: return keyboard report from update
    }
}
