//! RGB LEDs under the keys

use embedded_hal::timer::CountDown as _;
use rp_pico::{
    hal::{
        fugit::ExtU32,
        gpio::{bank0::Gpio2, FunctionPio0, Pin, PullDown},
        pio::SM0,
        timer::{CountDown, Instant},
    },
    pac::PIO0,
};
use smart_leds::brightness;
use smart_leds_trait::{SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

const RGB_LEN: usize = 12;
const FRAME_TIME: u32 = 33;

pub struct RgbMod<'timer> {
    ws: Ws2812<PIO0, SM0, CountDown<'timer>, Pin<Gpio2, FunctionPio0, PullDown>>,
    brightness: u8,
    buffer: [RGB8; RGB_LEN],
    frame_timer: CountDown<'timer>,
}

impl<'timer> RgbMod<'timer> {
    pub fn new(
        ws: Ws2812<PIO0, SM0, CountDown<'timer>, Pin<Gpio2, FunctionPio0, PullDown>>,
        mut count_down: CountDown<'timer>,
    ) -> Self {
        count_down.start(FRAME_TIME.millis());

        RgbMod {
            ws: ws,
            brightness: 32,
            buffer: [(0, 0, 0).into(); RGB_LEN],
            frame_timer: count_down,
        }
    }

    pub fn update(&mut self, t: Instant) {
        if !self.frame_timer.wait().is_ok() {
            return;
        }

        let t = ((t.duration_since_epoch().ticks() >> 13) % 360) as f32;

        for (i, led) in self.buffer.iter_mut().enumerate() {
            *led = hsv2rgb((t + (10 * (RGB_LEN - i)) as f32) % 360.0, 1.0, 1.0).into();
        }

        self.ws
            .write(brightness(self.buffer.iter().copied(), self.brightness))
            .unwrap();
    }
}

pub fn hsv2rgb(hue: f32, sat: f32, val: f32) -> (u8, u8, u8) {
    let c = val * sat;
    let v = (hue / 60.0) % 2.0 - 1.0;
    let v = if v < 0.0 { -v } else { v };
    let x = c * (1.0 - v);
    let m = val - c;
    let (r, g, b) = if hue < 60.0 {
        (c, x, 0.0)
    } else if hue < 120.0 {
        (x, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x)
    } else if hue < 240.0 {
        (0.0, x, c)
    } else if hue < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;

    (r, g, b)
}
