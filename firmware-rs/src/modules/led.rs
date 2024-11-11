//! Blinken Light for debugging module

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{bank0::Gpio25, FunctionSio, Pin, PullDown, SioOutput},
    timer::CountDown,
};

const BLINK_TIME: u32 = 500;

pub struct LedMod<'timer> {
    led_pin: Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
    timer: CountDown<'timer>,
    led_on: bool,
}

impl<'timer> LedMod<'timer> {
    pub fn new(
        led_pin: Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
        mut count_down: CountDown<'timer>,
    ) -> Self {
        count_down.start(BLINK_TIME.millis());

        LedMod {
            led_pin: led_pin,
            timer: count_down,
            led_on: true,
        }
    }

    pub fn clear(&mut self) {
        self.led_pin.set_low().unwrap();
    }

    pub fn update(&mut self) {
        if !self.timer.wait().is_ok() {
            return;
        }

        if self.led_on {
            self.led_pin.set_high().unwrap();
        } else {
            self.led_pin.set_low().unwrap();
        }

        self.led_on = !self.led_on;
    }
}
