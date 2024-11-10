//! Keyboard processing module

use crate::util;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
    timer::CountDown,
    usb::UsbBus,
};
use usbd_hid::{device::keyboard::NKROBootKeyboard, page::Keyboard, UsbHidError};
use usbd_human_interface_device as usbd_hid;

const POLL_RATE: u32 = 5;
const KEY_ROWS: usize = 3;
const KEY_COLS: usize = 4;

const KEY_MAP: [Keyboard; 12] = [
    Keyboard::F13,
    Keyboard::F14,
    Keyboard::F15,
    Keyboard::F16,
    Keyboard::F17,
    Keyboard::F18,
    Keyboard::F19,
    Keyboard::F20,
    Keyboard::F21,
    Keyboard::F22,
    Keyboard::F23,
    Keyboard::F24,
];

pub struct KeyboardMod<'timer> {
    col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
    row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
    poll_timer: CountDown<'timer>,
    pressed_keys: [Keyboard; 12],
}

impl<'timer> KeyboardMod<'timer> {
    pub fn new(
        col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
        row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
        mut count_down: CountDown<'timer>,
    ) -> Self {
        count_down.start(POLL_RATE.millis());

        KeyboardMod {
            col_pins: col_pins,
            row_pins: row_pins,
            poll_timer: count_down,
            pressed_keys: [Keyboard::NoEventIndicated; 12],
        }
    }

    fn check_pressed_keys(&mut self) -> [Keyboard; 12] {
        let mut keys = [Keyboard::NoEventIndicated; 12];

        for row in 0..KEY_ROWS {
            self.row_pins[row].set_high().unwrap();
            util::nop_loop(30);

            for col in 0..KEY_COLS {
                if self.col_pins[col].is_high().unwrap() {
                    let i = row * KEY_COLS + col;
                    keys[i] = KEY_MAP[i];
                }
            }

            self.row_pins[row].set_low().unwrap();
        }

        keys
    }

    fn update_keys(&mut self) {
        let new_keys = self.check_pressed_keys();
        self.pressed_keys = new_keys;
    }

    pub fn get_pressed_keys(&self) -> [Keyboard; 12] {
        self.pressed_keys
    }

    pub fn update(&mut self, hid: &mut NKROBootKeyboard<UsbBus>) {
        if !self.poll_timer.wait().is_ok() {
            return;
        }

        self.update_keys();
        match hid.write_report(self.pressed_keys) {
            Ok(_) => {}
            Err(UsbHidError::Duplicate) => {}
            Err(UsbHidError::WouldBlock) => {}
            Err(e) => {
                panic!("Failed to process keyboard tick: {:?}", e)
            }
        }
    }
}
