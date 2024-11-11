//! Keyboard processing module

use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
    timer::CountDown,
};

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

    fn check_pressed_keys(&mut self) -> [bool; 12] {
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

        keys
    }

    fn update_keys(&mut self) {
        let new_keys = self.check_pressed_keys();
        self.pressed_keys = new_keys;
    }

    pub fn update(&mut self) {
        if !self.poll_timer.wait().is_ok() {
            return;
        }

        self.update_keys();
    }
}

fn nop_loop(n: u8) {
    for _n in 0..n {
        cortex_m::asm::nop();
    }
}
