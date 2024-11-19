//! Screen for fun graphics

use defmt::info;
use embedded_hal::timer::CountDown as _;
use rp_pico::{
    hal::{
        fugit::ExtU32,
        gpio::{DynPinId, FunctionPio1, Pin, PullDown},
        pio::SM1,
        timer::{CountDown, Instant},
        Timer,
    },
    pac::PIO1,
};

use crate::{
    color::{hsv2rgb, rgb565},
    st7789::St7789,
};

const REFRESH_RATE: u32 = 50;

pub struct ScreenMod<'timer> {
    st: St7789<'timer, PIO1, SM1, Pin<DynPinId, FunctionPio1, PullDown>>,
    timer: CountDown<'timer>,
}

impl<'timer> ScreenMod<'timer> {
    pub fn new(
        st: St7789<'timer, PIO1, SM1, Pin<DynPinId, FunctionPio1, PullDown>>,
        mut count_down: CountDown<'timer>,
    ) -> Self {
        count_down.start(REFRESH_RATE.millis());

        ScreenMod {
            st: st,
            timer: count_down,
        }
    }

    pub fn clear(&mut self) {
        self.st.clear_framebuffer();
        self.st.push_framebuffer();
        self.st.backlight_off();
    }

    pub fn update(&mut self, t: Instant, timer: &Timer) {
        if !self.timer.wait().is_ok() {
            return;
        }

        let t = ((t.duration_since_epoch().ticks() >> 15) % 360) as f32;
        let rgb = hsv2rgb(t, 1.0, 1.0);
        let rgb = rgb565(rgb.0, rgb.1, rgb.2);

        // let time_start = timer.get_counter();
        self.st.fill_framebuffer(rgb);
        // let elapse1 = (timer.get_counter() - time_start).to_micros();

        // let time_start = timer.get_counter();
        self.st.push_framebuffer();
        // let elapse2 = (timer.get_counter() - time_start).to_micros();

        // info!("times: fill-fb={}us, write-fb={}us", elapse1, elapse2);
    }
}
