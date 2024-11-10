//! Firmware for JukeBox

#![no_std]
#![no_main]

mod uid;
mod util;
mod modules {
    pub mod keyboard;
    pub mod led;
    pub mod rgb;
    pub mod screen;
    pub mod serial;
}

use modules::*;

use embedded_hal::timer::CountDown as _;
use panic_probe as _;
use rp_pico::hal::{
    clocks::init_clocks_and_plls,
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PinState, PullDown},
    multicore::{Multicore, Stack},
    pac::Peripherals,
    pio::PIOExt,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Clock, Timer,
};
use rp_pico::{entry, Pins};
use ws2812_pio::Ws2812;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::device::{
    keyboard::NKROBootKeyboard,
    // mouse::{WheelMouse, WheelMouseReport},
};
use usbd_hid::prelude::*;
use usbd_human_interface_device as usbd_hid;
use usbd_serial::SerialPort;

#[allow(unused_imports)]
use defmt::*;
use defmt_rtt as _;

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
    // load unique flash id
    let ver = env!("CARGO_PKG_VERSION");
    let uid = uid::get_flash_uid();
    info!("ver:{}, uid:{}", ver, uid);

    // set up hardware interfaces
    let mut pac = Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let mut sio = Sio::new(pac.SIO);
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let core1 = &mut mc.cores()[1];

    // set up timers
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut serial_timer = timer.count_down();
    serial_timer.start(100.millis());
    let mut hid_tick = timer.count_down();
    hid_tick.start(4.millis());
    let mut nkro_tick = timer.count_down();
    nkro_tick.start(1.millis());

    // set up usb
    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut usb_hid = UsbHidClassBuilder::new()
        .add_device(usbd_hid::device::keyboard::NKROBootKeyboardConfig::default())
        // .add_device(usbd_hid::device::mouse::WheelMouseConfig::default())
        .build(&usb_bus);
    let mut usb_serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0xF20A))
        .strings(&[StringDescriptors::default()
            .manufacturer("FriendTeamInc")
            .product("JukeBox V5")
            .serial_number(&uid)])
        .unwrap()
        .composite_with_iads()
        .build();

    // set up inter-core mutexes

    // set up modules
    let mut serial_mod = serial::SerialMod::new(timer.count_down());

    // core 1 event loop (GPIO)
    let _ = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        let mut pac = unsafe { Peripherals::steal() };
        // let core = unsafe { CorePeripherals::steal() };
        // let sio = Sio::new(pac.SIO);
        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // set up GPIO
        let kb_col_pins: [Pin<DynPinId, FunctionSioInput, PullDown>; 4] = [
            pins.gpio12.into_pull_down_input().into_dyn_pin(),
            pins.gpio13.into_pull_down_input().into_dyn_pin(),
            pins.gpio14.into_pull_down_input().into_dyn_pin(),
            pins.gpio15.into_pull_down_input().into_dyn_pin(),
        ];
        let kb_row_pins: [Pin<DynPinId, FunctionSioOutput, PullDown>; 3] = [
            pins.gpio9
                .into_push_pull_output_in_state(PinState::Low)
                .into_dyn_pin(),
            pins.gpio10
                .into_push_pull_output_in_state(PinState::Low)
                .into_dyn_pin(),
            pins.gpio11
                .into_push_pull_output_in_state(PinState::Low)
                .into_dyn_pin(),
        ];

        let led_pin = pins.led.into_push_pull_output();

        let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
        let ws = Ws2812::new(
            pins.gpio2.into_function(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
            timer.count_down(),
        );

        // set up modules
        let mut keyboard_mod =
            keyboard::KeyboardMod::new(kb_col_pins, kb_row_pins, timer.count_down());

        let mut led_mod = led::LedMod::new(led_pin, timer.count_down());
        let mut rgb_mod = rgb::RgbMod::new(ws, timer.count_down());
        let mut screen_mod = screen::ScreenMod::new();

        loop {
            // update input devices
            keyboard_mod.update();

            // update accessories
            led_mod.update();
            rgb_mod.update(timer.get_counter());
            screen_mod.update();
        }
    });

    // main event loop (USB comms)
    loop {
        // tick for hid devices
        if hid_tick.wait().is_ok() {
            // handle keyboard
            match usb_hid
                .device::<NKROBootKeyboard<'_, _>, _>()
                .write_report([usbd_hid::page::Keyboard::NoEventIndicated; 12])
            {
                Ok(_) => {}
                Err(UsbHidError::Duplicate) => {}
                Err(UsbHidError::WouldBlock) => {}
                Err(e) => {
                    core::panic!("Failed to write keyboard report: {:?}", e)
                }
            }

            // handle mouse
            // match usb_hid
            //     .device::<WheelMouse<'_, _>, _>()
            //     .write_report(&WheelMouseReport::default())
            // {
            //     Ok(_) => {}
            //     Err(UsbHidError::Duplicate) => {}
            //     Err(UsbHidError::WouldBlock) => {}
            //     Err(e) => {
            //         core::panic!("Failed to write mouse report: {:?}", e)
            //     }
            // }
        }

        // tick for n-key rollover
        if nkro_tick.wait().is_ok() {
            match usb_hid.tick() {
                Ok(_) => {}
                Err(UsbHidError::WouldBlock) => {}
                Err(e) => {
                    core::panic!("Failed to process keyboard tick: {:?}", e)
                }
            };
        }

        // update usb devices
        if usb_dev.poll(&mut [&mut usb_hid, &mut usb_serial]) {
            // handle serial
            serial_mod.update(&mut usb_serial, ver, uid);
        }
    }
}
