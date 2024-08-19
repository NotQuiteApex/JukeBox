//! Keyboard processing module

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
    timer::CountDown,
    usb::UsbBus,
    Timer,
};
use usbd_hid::{
    descriptor::{KeyboardReport, KeyboardUsage::*},
    hid_class::HIDClass,
};

use crate::util;

const POLL_RATE: u32 = 10;
const KEY_ROWS: usize = 3;
const KEY_COLS: usize = 4;

const KEY_MAP: [u8; 12] = [
    KeyboardF13 as u8,
    KeyboardF14 as u8,
    KeyboardF15 as u8,
    KeyboardF16 as u8,
    KeyboardF17 as u8,
    KeyboardF18 as u8,
    KeyboardF19 as u8,
    KeyboardF20 as u8,
    KeyboardF21 as u8,
    KeyboardF22 as u8,
    KeyboardF23 as u8,
    KeyboardF24 as u8,
];

pub struct KeyboardMod<'a> {
    col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
    row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
    timer: CountDown<'a>,
    prev_keys: [u8; 12],
}

impl<'timer> KeyboardMod<'timer> {
    pub fn new(
        col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
        row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
        timer: &'timer Timer,
    ) -> Self {
        let count_down = timer.count_down();

        let mut this = KeyboardMod {
            col_pins: col_pins,
            row_pins: row_pins,
            timer: count_down,
            prev_keys: [0u8; 12],
        };

        this.timer.start(POLL_RATE.millis());

        this
    }

    fn get_pressed_keys(&mut self) -> [u8; 12] {
        let mut keys = [0u8; 12];

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

    pub fn update(&mut self, hid: &mut HIDClass<UsbBus>) {
        let new_keys = self.get_pressed_keys();

        let mut keycodes = [0u8; 6];
        let mut keycode_count = 0usize;
        for k in new_keys {
            if keycode_count >= 6 {
                break;
            }
            if k != 0 {
                keycodes[keycode_count] = k;
                keycode_count += 1;
            }
        }
        let rep = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: keycodes,
        };
        let res = hid.push_input(&rep);

        self.prev_keys = new_keys;

        // TODO: implement nkro from https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/keyboard_nkro.rs
        // TODO: return keyboard report from update
    }
}
