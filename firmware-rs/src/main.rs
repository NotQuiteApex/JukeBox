//! Firmware for JukeBox

#![no_std]
#![no_main]

mod uid;
mod modules {
    pub mod keyboard;
    pub mod led;
    pub mod serial;
}

use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
    clocks::init_clocks_and_plls, pac::Peripherals, sio::Sio, usb, watchdog::Watchdog, Timer,
};
use panic_probe as _;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;
use usbd_serial::SerialPort;

// use defmt::*;
// use defmt_rtt as _;

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

    // set up timers
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // set up usb
    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 10);
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
    let mut led_mod = modules::led::LedMod::new(led_pin, &timer);
    let mut keyboard_mod = modules::keyboard::KeyboardMod::new();
    let mut serial_mod = modules::serial::SerialMod::new();

    // main event loop
    loop {
        if usb_dev.poll(&mut [&mut usb_hid, &mut usb_serial]) {
            keyboard_mod.update(&mut usb_hid);
            serial_mod.update(&mut usb_serial);
        }

        led_mod.update();
    }
}
