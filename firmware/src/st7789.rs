use core::u32;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::digital::v2::OutputPin as _;
use rp_pico::hal::{
    fugit::{ExtU64, MicrosDurationU64},
    gpio::{AnyPin, DynPinId, FunctionSioOutput, Pin, PullDown},
    pio::{PIOBuilder, PIOExt, StateMachineIndex, Tx, UninitStateMachine, PIO},
    timer::CountDown,
};

const SCR_W: usize = 240;
const SCR_H: usize = 320;
static mut FB: [[u16; SCR_W]; SCR_H] = [[0x00FFu16; SCR_W]; SCR_H];
// The framebuffer is a static so that it does not end up on core1's stack.

pub struct St7789<'timer, P, SM, I>
where
    I: AnyPin<Function = P::PinFunction>,
    SM: StateMachineIndex,
    P: PIOExt,
{
    tx: Tx<(P, SM)>,
    _data_pin: I,
    _clock_pin: I,
    backlight_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
    dc_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
    cs_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
    _rst_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
    timer: CountDown<'timer>,
}

impl<'timer, P, SM, I> St7789<'timer, P, SM, I>
where
    I: AnyPin<Function = P::PinFunction>,
    P: PIOExt,
    SM: StateMachineIndex,
{
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        data_pin: I,
        clock_pin: I,
        mut cs_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
        mut dc_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
        mut rst_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
        mut backlight_pin: Pin<DynPinId, FunctionSioOutput, PullDown>,
        timer: CountDown<'timer>,
    ) -> Self {
        backlight_pin.set_low().unwrap();
        dc_pin.set_low().unwrap();
        cs_pin.set_high().unwrap();
        rst_pin.set_high().unwrap();

        let side_set = pio::SideSet::new(false, 1, false);
        let mut a = pio::Assembler::new_with_side_set(side_set);
        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        a.bind(&mut wrap_target);
        a.out_with_side_set(pio::OutDestination::PINS, 1, 0);
        a.nop_with_side_set(1);
        a.bind(&mut wrap_source);
        let program = a.assemble_with_wrap(wrap_source, wrap_target);
        let installed = pio.install(&program).unwrap();

        let data_pin = data_pin.into();
        let clock_pin = clock_pin.into();
        let (mut sm, _, tx) = PIOBuilder::from_installed_program(installed)
            // pin config
            .side_set_pin_base(clock_pin.id().num)
            .out_pins(data_pin.id().num, 1)
            // buffer config
            .buffers(rp_pico::hal::pio::Buffers::OnlyTx)
            .out_shift_direction(rp_pico::hal::pio::ShiftDirection::Left)
            .autopull(true)
            .pull_threshold(16)
            // misc config
            .clock_divisor_fixed_point(1, 0)
            .build(sm);

        sm.set_pindirs([
            (data_pin.id().num, rp_pico::hal::pio::PinDir::Output),
            (clock_pin.id().num, rp_pico::hal::pio::PinDir::Output),
        ]);

        sm.start();

        Self {
            tx: tx,
            _data_pin: data_pin.into(),
            _clock_pin: clock_pin.into(),
            backlight_pin: backlight_pin,
            dc_pin: dc_pin,
            cs_pin: cs_pin,
            _rst_pin: rst_pin,
            timer: timer,
        }
    }

    pub fn backlight_on(&mut self) {
        self.backlight_pin.set_high().unwrap();
    }

    pub fn backlight_off(&mut self) {
        self.backlight_pin.set_low().unwrap();
    }

    pub fn init(&mut self) {
        // init sequence
        self.write_cmd(&[0x01]); // Software reset
        self.write_cmd(&[0x11]); // Exit sleep mode
        self.write_cmd(&[0x3A, 0x55]); // Set colour mode to 16 bit
        self.write_cmd(&[0x36, 0x00]); // Set MADCTL: row then column, refresh is bottom to top ????
        self.write_cmd(&[0x2A, 0x00, SCR_W as u16]); // CASET: column addresses
        self.write_cmd(&[0x2B, 0x00, SCR_H as u16]); // RASET: row addresses
        self.write_cmd(&[0x21]); // Inversion on (supposedly a hack?)
        self.write_cmd(&[0x13]); // Normal display on
        self.write_cmd(&[0x29]); // Main screen turn on

        self.push_framebuffer();
        self.backlight_on();
    }

    fn wait_idle(&mut self) {
        self.tx.clear_stalled_flag();
        while !self.tx.has_stalled() {}
    }

    fn sleep(&mut self, t: MicrosDurationU64) {
        self.timer.start(t);
        loop {
            match self.timer.wait() {
                Ok(_) => break,
                Err(_) => {}
            }
        }
    }

    fn write(&mut self, word: u16) {
        while !self.tx.write((word as u32) << 16) {
            cortex_m::asm::nop();
        }
    }

    fn write_cmd(&mut self, cmd: &[u16]) {
        self.wait_idle();
        self.set_dc_cs(false, false);

        self.write(cmd[0]);
        if cmd.len() >= 2 {
            self.wait_idle();
            self.set_dc_cs(true, false);
            for c in &cmd[1..] {
                self.write(*c);
            }
        }

        self.wait_idle();
        self.set_dc_cs(true, true);
    }

    fn set_dc_cs(&mut self, dc: bool, cs: bool) {
        self.sleep(1.micros().into());

        if dc {
            self.dc_pin.set_high().unwrap();
        } else {
            self.dc_pin.set_low().unwrap();
        }
        if cs {
            self.cs_pin.set_high().unwrap();
        } else {
            self.cs_pin.set_low().unwrap();
        }

        self.sleep(1.micros().into());
    }

    fn start_pixels(&mut self) {
        self.write_cmd(&[0x002C]);
        self.set_dc_cs(true, false);
    }

    pub fn fill_framebuffer(&mut self, color: u16) {
        for y in 0..SCR_H {
            for x in 0..SCR_W {
                // doing unchecked access did not meaningfully improve performance
                unsafe {
                    FB[y][x] = color;
                }
            }
        }
    }

    pub fn clear_framebuffer(&mut self) {
        self.fill_framebuffer(0);
    }

    pub fn push_framebuffer(&mut self) {
        self.start_pixels();
        for y in 0..SCR_H {
            let y = unsafe { FB.get_unchecked(y) };
            for x in 0..SCR_W {
                let w = unsafe { y.get_unchecked(x) };
                self.write(*w);
                // let w1 = (*w >> 8) as u8;
                // let w2 = *w as u8;

                // self.write(w1);
                // self.write(w2);
            }
        }
    }
}
