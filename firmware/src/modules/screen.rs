//! Screen for fun graphics

use embedded_hal::timer::CountDown as _;
use rp_pico::hal::{fugit::ExtU32, timer::CountDown};

const REFRESH_RATE: u32 = 33;

pub struct ScreenMod<'timer> {
    timer: CountDown<'timer>,
}

impl<'timer> ScreenMod<'timer> {
    pub fn new(mut count_down: CountDown<'timer>) -> Self {
        count_down.start(REFRESH_RATE.millis());

        ScreenMod { timer: count_down }
    }

    pub fn clear(&mut self) {}

    pub fn update(&mut self) {}
}
