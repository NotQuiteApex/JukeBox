use rp_pico::{
    hal::gpio::{DynPinId, FunctionPio0, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
    Pins,
};

use crate::keyboard::{KEY_COLS, KEY_ROWS};

pub fn configure_gpio(
    pins: Pins,
) -> (
    [Pin<DynPinId, FunctionSioInput, PullDown>; KEY_COLS],
    [Pin<DynPinId, FunctionSioOutput, PullDown>; KEY_ROWS],
    Pin<DynPinId, FunctionSioOutput, PullDown>,
    Pin<DynPinId, FunctionPio0, PullDown>,
) {
    (
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
        pins.led.into_function().into_dyn_pin().into_pull_type(),
        pins.gpio2.into_function().into_dyn_pin().into_pull_type(),
    )
}
