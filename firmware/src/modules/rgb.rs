//! RGB LEDs under the keys

use embedded_hal::timer::CountDown as _;
use rp_pico::{
    hal::{
        fugit::ExtU32,
        gpio::{DynPinId, FunctionPio0, Pin, PullDown},
        pio::SM0,
        timer::{CountDown, Instant},
    },
    pac::PIO0,
};
use smart_leds::brightness;
use smart_leds_trait::{SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

use crate::color::hsv2rgb;

const RGB_LEN: usize = 12;
const FRAME_TIME: u32 = 33;

pub struct RgbMod<'timer> {
    ws: Ws2812<PIO0, SM0, CountDown<'timer>, Pin<DynPinId, FunctionPio0, PullDown>>,
    brightness: u8,
    buffer: [RGB8; RGB_LEN],
    timer: CountDown<'timer>,
}

impl<'timer> RgbMod<'timer> {
    pub fn new(
        ws: Ws2812<PIO0, SM0, CountDown<'timer>, Pin<DynPinId, FunctionPio0, PullDown>>,
        mut count_down: CountDown<'timer>,
    ) -> Self {
        count_down.start(FRAME_TIME.millis());

        RgbMod {
            ws: ws,
            brightness: 10,
            buffer: [(0, 0, 0).into(); RGB_LEN],
            timer: count_down,
        }
    }

    pub fn clear(&mut self) {
        self.brightness = 0;
        self.buffer = [(0, 0, 0).into(); RGB_LEN];
        self.ws
            .write(brightness(self.buffer.iter().copied(), self.brightness))
            .unwrap();
    }

    pub fn update(&mut self, t: Instant) {
        if !self.timer.wait().is_ok() {
            return;
        }

        let t = ((t.duration_since_epoch().ticks() >> 12) % 360) as f32;

        for (i, led) in self.buffer.iter_mut().enumerate() {
            *led = hsv2rgb((t + (10 * (RGB_LEN - i)) as f32) % 360.0, 1.0, 1.0).into();
        }

        self.ws
            .write(brightness(self.buffer.iter().copied(), self.brightness))
            .unwrap();
    }
}
