//! Firmware for JukeBox

#![no_std]
#![no_main]

mod modules {
    pub mod led;
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

use defmt::*;
use defmt_rtt as _;

#[entry]
fn main() -> ! {
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

    // set up usb interfaces
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
            .serial_number("SERIAL_NO")])
        .unwrap()
        .device_class(0xEF)
        .device_sub_class(0x02)
        .device_protocol(0x01)
        .build();

    // set up modules
    let mut led_mod = modules::led::LedMod::new(led_pin, &timer);

    // main event loop
    loop {
        led_mod.update();

        if usb_dev.poll(&mut [&mut usb_serial, &mut usb_hid]) {
            // let mut buf = [0u8; 64];
        }
    }
}
