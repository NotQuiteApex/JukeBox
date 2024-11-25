use jukebox_util::peripheral::{JBInputs, KeyInputs, KnobInputs, PedalInputs};
use rp_pico::hal::usb::UsbBus;
use usbd_serial::SerialPort;

pub const fn inputs_default() -> JBInputs {
    if cfg!(feature = "keypad") {
        JBInputs::KeyPad(KeyInputs::default())
    } else if cfg!(feature = "knobpad") {
        JBInputs::KnobPad(KnobInputs::default())
    } else if cfg!(feature = "pedalpad") {
        JBInputs::PedalPad(PedalInputs::default())
    } else {
        JBInputs::KeyPad(KeyInputs::default())
    }
}

pub fn inputs_write_report(inputs: JBInputs, serial: &mut SerialPort<UsbBus>) {
    let _ = match inputs {
        JBInputs::KeyPad(i) => serial.write(&i.encode()),
        JBInputs::KnobPad(i) => serial.write(&i.encode()),
        JBInputs::PedalPad(i) => serial.write(&i.encode()),
    };
}
