//! Firmware for JukeBox

#![no_std]
#![no_main]

mod uid;
mod util;
mod modules {
    pub mod keyboard;
    pub mod led;
    pub mod serial;
}

use modules::*;

use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
    clocks::init_clocks_and_plls,
    fugit::ExtU32,
    gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PinState, PullDown},
    pac::Peripherals,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Timer,
};
use embedded_hal::timer::CountDown as _;
use panic_probe as _;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::prelude::*;
use usbd_human_interface_device as usbd_hid;
use usbd_serial::SerialPort;

use defmt::*;
use defmt_rtt as _;

#[entry]
fn main() -> ! {
    // load unique flash id
    let uid = uid::get_flash_uid();

    // set up hardware interfaces
    let mut pac = Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let sio = Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // set up hardware pins
    let led_pin = pins.led.into_push_pull_output();
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

    // set up timers
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut tick_timer = timer.count_down();
    tick_timer.start(1.millis());

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

    // set up modules
    let mut led_mod = led::LedMod::new(led_pin, &timer);
    let mut keyboard_mod = keyboard::KeyboardMod::new(kb_col_pins, kb_row_pins, &timer);
    let mut serial_mod = serial::SerialMod::new();

    // main event loop
    loop {
        // tick for n-key rollover
        // TODO: move to keyboard module for cleanliness?
        if tick_timer.wait().is_ok() {
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
            keyboard_mod.update(&mut usb_hid.device());
            serial_mod.update(&mut usb_serial);
        }

        // update peripherals
        led_mod.update();
    }
}
