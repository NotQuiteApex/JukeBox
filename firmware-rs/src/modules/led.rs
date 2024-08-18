//! Blinken Light module

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{
    fugit::ExtU32,
    gpio::{bank0::Gpio25, FunctionSio, Pin, PullDown, SioOutput},
    timer::CountDown,
    Timer,
};

const BLINK_TIME: u32 = 500;

pub struct LedMod<'a> {
    led_pin: Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
    timer: CountDown<'a>,
    led_on: bool,
}

impl<'timer> LedMod<'timer> {
    pub fn new(
        led_pin: Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
        timer: &'timer Timer,
    ) -> Self {
        let count_down = timer.count_down();

        let mut this = LedMod {
            led_pin: led_pin,
            timer: count_down,
            led_on: false,
        };

        this.timer_start();

        this
    }

    fn timer_start(&mut self) {
        self.timer.start(BLINK_TIME.millis());
    }

    pub fn update(&mut self) {
        if self.timer.wait().is_ok() {
            if self.led_on {
                self.led_pin.set_high().unwrap();
            } else {
                self.led_pin.set_low().unwrap();
            }

            self.led_on = !self.led_on;
        }
    }
}
