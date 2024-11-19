use rp_pico::{
    hal::gpio::{
        DynPinId, FunctionPio0, FunctionPio1, FunctionSioInput, FunctionSioOutput, Pin, PullDown,
    },
    Pins,
};

use crate::keyboard::{KEY_COLS, KEY_ROWS};

pub fn configure_gpio(
    pins: Pins,
) -> (
    // keyboard keys
    [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
    [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
    // led pin
    Pin<DynPinId, FunctionSioOutput, PullDown>,
    // rgb pin
    Pin<DynPinId, FunctionPio0, PullDown>,
    // screen pins
    (
        Pin<DynPinId, FunctionPio1, PullDown>,      // data
        Pin<DynPinId, FunctionPio1, PullDown>,      // clock
        Pin<DynPinId, FunctionSioOutput, PullDown>, // cs
        Pin<DynPinId, FunctionSioOutput, PullDown>, // dc
        Pin<DynPinId, FunctionSioOutput, PullDown>, // rst
        Pin<DynPinId, FunctionSioOutput, PullDown>, // backlight
    ),
) {
    (
        // keyboard keys
        [
            pins.gpio12.into_function().into_dyn_pin().into_pull_type(),
            pins.gpio13.into_function().into_dyn_pin().into_pull_type(),
            pins.gpio14.into_function().into_dyn_pin().into_pull_type(),
            pins.gpio15.into_function().into_dyn_pin().into_pull_type(),
        ],
        [
            pins.gpio9.into_function().into_dyn_pin().into_pull_type(),
            pins.gpio10.into_function().into_dyn_pin().into_pull_type(),
            pins.gpio11.into_function().into_dyn_pin().into_pull_type(),
        ],
        // led pin
        pins.led.into_function().into_dyn_pin().into_pull_type(),
        // rgb pin
        pins.gpio2.into_function().into_dyn_pin().into_pull_type(),
        // screen pins
        (
            pins.gpio21.into_function().into_dyn_pin().into_pull_type(), // data
            pins.gpio20.into_function().into_dyn_pin().into_pull_type(), // clock
            pins.gpio19.into_function().into_dyn_pin().into_pull_type(), // cs
            pins.gpio18.into_function().into_dyn_pin().into_pull_type(), // dc
            pins.gpio17.into_function().into_dyn_pin().into_pull_type(), // rst
            pins.gpio16.into_function().into_dyn_pin().into_pull_type(), // backlight
        ),
    )
}
