//! Firmware for JukeBox

#![no_std]
#![no_main]

use jukebox_util::peripheral::JBInputs;
use mutually_exclusive_features::exactly_one_of;
exactly_one_of!("keypad", "knobpad", "pedalpad");

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

mod mutex;
mod peripheral;
mod st7789;
mod uid;
mod modules {
    #[cfg(feature = "keypad")]
    pub mod keyboard;
    pub mod led;
    pub mod rgb;
    #[cfg(feature = "keypad")]
    pub mod screen;
    pub mod serial;
}

use modules::*;
use mutex::Mutex;

use embedded_hal::timer::CountDown as _;
use panic_probe as _;
use peripheral::inputs_default;
use rp_pico::hal::{
    clocks::init_clocks_and_plls,
    fugit::ExtU32,
    multicore::{Multicore, Stack},
    pac::Peripherals,
    pio::PIOExt,
    rom_data::reset_to_usb_boot,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Clock, Timer,
};
use rp_pico::{entry, Pins};

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::prelude::*;
use usbd_human_interface_device as usbd_hid;
use usbd_serial::SerialPort;

#[allow(unused_imports)]
use defmt::*;
use defmt_rtt as _;

static mut CORE1_STACK: Stack<8192> = Stack::new();

// inter-core mutexes
static PERIPHERAL_INPUTS: Mutex<1, JBInputs> = Mutex::new(inputs_default());
static UPDATE_TRIGGER: Mutex<2, bool> = Mutex::new(false);

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
    let usb_pid = if cfg!(feature = "keypad") {
        0xF20A
    } else if cfg!(feature = "knobpad") {
        0xF20B
    } else if cfg!(feature = "pedalpad") {
        0xF20C
    } else {
        0xF209
    };
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, usb_pid))
        .strings(&[StringDescriptors::default()
            .manufacturer("FriendTeamInc")
            .product("JukeBox V5")
            .serial_number(&uid)])
        .unwrap()
        .composite_with_iads()
        .build();

    // set up modules
    let mut serial_mod = serial::SerialMod::new(timer.count_down());

    // core 1 event loop (GPIO)
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            let mut pac = unsafe { Peripherals::steal() };
            let pins = Pins::new(
                pac.IO_BANK0,
                pac.PADS_BANK0,
                sio.gpio_bank0,
                &mut pac.RESETS,
            );

            // set up GPIO and modules
            #[cfg(feature = "keypad")]
            let mut keyboard_mod = {
                let kb_col_pins = [
                    pins.gpio12.into_function().into_dyn_pin().into_pull_type(),
                    pins.gpio13.into_function().into_dyn_pin().into_pull_type(),
                    pins.gpio14.into_function().into_dyn_pin().into_pull_type(),
                    pins.gpio15.into_function().into_dyn_pin().into_pull_type(),
                ];
                let kb_row_pins = [
                    pins.gpio9.into_function().into_dyn_pin().into_pull_type(),
                    pins.gpio10.into_function().into_dyn_pin().into_pull_type(),
                    pins.gpio11.into_function().into_dyn_pin().into_pull_type(),
                ];
                keyboard::KeyboardMod::new(kb_col_pins, kb_row_pins, timer.count_down())
            };

            #[cfg(feature = "keypad")]
            let mut screen_mod = {
                let screen_pins = (
                    pins.gpio21.into_function().into_dyn_pin().into_pull_type(), // data
                    pins.gpio20.into_function().into_dyn_pin().into_pull_type(), // clock
                    pins.gpio19.into_function().into_dyn_pin().into_pull_type(), // cs
                    pins.gpio18.into_function().into_dyn_pin().into_pull_type(), // dc
                    pins.gpio17.into_function().into_dyn_pin().into_pull_type(), // rst
                    pins.gpio16.into_function().into_dyn_pin().into_pull_type(), // backlight
                );
                let (mut pio1, _, sm1, _, _) = pac.PIO1.split(&mut pac.RESETS);
                let mut st = st7789::St7789::new(
                    &mut pio1,
                    sm1,
                    screen_pins.0,
                    screen_pins.1,
                    screen_pins.2,
                    screen_pins.3,
                    screen_pins.4,
                    screen_pins.5,
                    timer.count_down(),
                );
                st.init();
                screen::ScreenMod::new(st, timer.count_down())
            };

            let mut led_mod = {
                let led_pin = pins.led.into_function().into_dyn_pin().into_pull_type();
                led::LedMod::new(led_pin, timer.count_down())
            };
            let mut rgb_mod = {
                let rgb_pin = pins.gpio2.into_function().into_dyn_pin().into_pull_type();
                let (mut pio0, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
                let ws = ws2812_pio::Ws2812::new(
                    rgb_pin,
                    &mut pio0,
                    sm0,
                    clocks.peripheral_clock.freq(),
                    timer.count_down(),
                );
                rgb::RgbMod::new(ws, timer.count_down())
            };

            loop {
                // update input devices
                #[cfg(feature = "keypad")]
                keyboard_mod.update();

                // update mutexes
                PERIPHERAL_INPUTS.with_mut_lock(|i| {
                    #[cfg(feature = "keypad")]
                    {
                        *i = JBInputs::KeyPad(keyboard_mod.get_pressed_keys().into());
                    }
                });

                // check if we need to shutdown "cleanly" for update
                UPDATE_TRIGGER.with_lock(|u| {
                    if *u {
                        #[cfg(feature = "keypad")]
                        screen_mod.clear();

                        led_mod.clear();
                        rgb_mod.clear();

                        // wait a few cycles for the IO to finish
                        for _ in 0..100 {
                            cortex_m::asm::nop();
                        }

                        reset_to_usb_boot(0, 0);
                    }
                });

                // update accessories
                led_mod.update();
                rgb_mod.update(timer.get_counter());

                #[cfg(feature = "keypad")]
                screen_mod.update(timer.get_counter(), &timer);
            }
        })
        .expect("failed to start core1");

    // main event loop (USB comms)
    loop {
        // tick for hid devices
        if hid_tick.wait().is_ok() {
            // handle keyboard
            // #[cfg(feature = "keypad")]
            // let pressed = keyboard::KeyboardMod::get_keyboard_keys(
            //     serial_mod.get_connection_status().connected(),
            //     &PERIPHERAL_INPUTS,
            // );
            // match usb_hid
            //     .device::<NKROBootKeyboard<'_, _>, _>()
            //     .write_report(pressed)
            // {
            //     Ok(_) => {}
            //     Err(UsbHidError::Duplicate) => {}
            //     Err(UsbHidError::WouldBlock) => {}
            //     Err(e) => {
            //         core::panic!("Failed to write keyboard report: {:?}", e)
            //     }
            // }

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
            serial_mod.update(
                &mut usb_serial,
                ver,
                uid,
                &PERIPHERAL_INPUTS,
                &UPDATE_TRIGGER,
            );
            match usb_serial.flush() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
