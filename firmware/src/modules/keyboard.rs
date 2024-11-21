//! Keyboard processing module

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
    timer::CountDown,
};
use usbd_human_interface_device::page::Keyboard;

use crate::mutex::Mutex;
use crate::peripheral::JBPeripheralInputs;

const POLL_RATE: u32 = 5;
pub const KEY_ROWS: usize = 3;
pub const KEY_COLS: usize = 4;

pub struct KeyboardMod<'timer> {
    col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
    row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
    poll_timer: CountDown<'timer>,
    pressed_keys: [bool; 12],
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
            pressed_keys: [false; 12],
        }
    }

    fn check_pressed_keys(&mut self) {
        let mut keys = [false; 12];

        for row in 0..KEY_ROWS {
            self.row_pins[row].set_high().unwrap();
            nop_loop(30);

            for col in 0..KEY_COLS {
                if self.col_pins[col].is_high().unwrap() {
                    let i = row * KEY_COLS + col;
                    keys[i] = true;
                }
            }

            self.row_pins[row].set_low().unwrap();
        }

        self.pressed_keys = keys;
    }

    pub fn update(&mut self) {
        if !self.poll_timer.wait().is_ok() {
            return;
        }

        self.check_pressed_keys();
    }

    pub fn get_pressed_keys(&self) -> [bool; 12] {
        self.pressed_keys
    }

    pub fn get_keyboard_keys(
        serial_connected: bool,
        peripheral_inputs: &Mutex<1, JBPeripheralInputs>,
    ) -> [Keyboard; 12] {
        let mut pressed = [Keyboard::NoEventIndicated; 12];
        if !serial_connected {
            let keys = [
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
            let mut i = [false; 12];
            peripheral_inputs.with_lock(|k| {
                i[0] = k.keyboard.key1.into();
                i[1] = k.keyboard.key2.into();
                i[2] = k.keyboard.key3.into();
                i[3] = k.keyboard.key4.into();
                i[4] = k.keyboard.key5.into();
                i[5] = k.keyboard.key6.into();
                i[6] = k.keyboard.key7.into();
                i[7] = k.keyboard.key8.into();
                i[8] = k.keyboard.key9.into();
                i[9] = k.keyboard.key10.into();
                i[10] = k.keyboard.key11.into();
                i[11] = k.keyboard.key12.into();
            });
            for (i, (k, j)) in keys.iter().zip(i).enumerate() {
                if j {
                    pressed[i] = *k;
                }
            }
        }

        pressed
    }
}

fn nop_loop(n: u8) {
    for _n in 0..n {
        cortex_m::asm::nop();
    }
}
