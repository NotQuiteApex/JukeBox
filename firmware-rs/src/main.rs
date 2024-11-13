//! Firmware for JukeBox

#![no_std]
#![no_main]

mod mutex;
mod peripheral;
mod pins;
mod uid;
mod modules {
    pub mod keyboard;
    pub mod led;
    pub mod rgb;
    pub mod screen;
    pub mod serial;
}

use modules::*;
use mutex::Mutex;

use embedded_hal::timer::CountDown as _;
use panic_probe as _;
use peripheral::{Connection, JBPeripheralInputs, JBPeripherals, SwitchPosition};
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

static mut CORE1_STACK: Stack<2048> = Stack::new();

// inter-core mutexes
static CONNECTED_PERIPHERALS: Mutex<0, JBPeripherals> = Mutex::new(JBPeripherals::default());
static PERIPHERAL_INPUTS: Mutex<1, JBPeripheralInputs> = Mutex::new(JBPeripheralInputs::default());
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
    // let mut usb_serial = SerialPort::new_with_store(&usb_bus, [0u8; 1024], [0u8; 1024]); // TODO ?
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0xF20A))
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
    let _ = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        let mut pac = unsafe { Peripherals::steal() };
        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // set up GPIO
        let (kb_col_pins, kb_row_pins, led_pin, rgb_pin) = pins::configure_gpio(pins);

        let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
        let ws = Ws2812::new(
            rgb_pin,
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

            // update mutexes
            CONNECTED_PERIPHERALS.with_mut_lock(|c| {
                c.keyboard = Connection::Connected;
            });
            PERIPHERAL_INPUTS.with_mut_lock(|i| {
                i.keyboard.key1 = SwitchPosition::Down;
            });

            // check if we need to shutdown "cleanly" for update
            UPDATE_TRIGGER.with_lock(|u| {
                if *u {
                    led_mod.clear();
                    rgb_mod.clear();
                    screen_mod.clear();

                    // wait a few cycles for the IO to finish
                    for _ in 0..100 {
                        cortex_m::asm::nop();
                    }

                    reset_to_usb_boot(0, 0);
                    // TODO: schedule a reset_to_usb_boot call,
                    // make sure to cleanly stop all other modules and clear screen and rgb.
                    core::todo!()
                }
            });
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
            serial_mod.update(
                &mut usb_serial,
                ver,
                uid,
                &CONNECTED_PERIPHERALS,
                &PERIPHERAL_INPUTS,
                &UPDATE_TRIGGER,
            );
            match usb_serial.flush() {
                Ok(_) => {}
                Err(_) => {
                    // warn!("flush failed!")
                }
            }
        }
    }
}
