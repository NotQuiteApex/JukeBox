# JukeBox Board
A board that exposes the keys F13-F24 for use with macros, or hotkeys for streaming (or anything, really)!

# Compatible programs
Sadly, not every program is compatible with these additional function keys, and some do not allow the use of these keys. There is a running list [here](docs/program-compatibility.md) of what programs do and do not work with the keys provided by JukeBox.

## Printed Circuit Board
Made with KiCad 6.0.7, the PCB is the physical heart of the project. See [board](board/) directory for the files. You can access the BOM [here](https://www.digikey.com/en/mylists/list/QC5ACPN0J3).

Footprints and model for USB4105-GF-A can be found [here](https://www.digikey.com/en/products/detail/gct/usb4105-gf-a/11198441). Footprints for Kailh Choc's can be found [here](https://github.com/daprice/keyswitches.pretty), model can be found [here](https://grabcad.com/library/kailh-low-profile-mechanical-keyboard-switch-1). The model for the Cherry MX Switch can be found [here](https://github.com/ConstantinoSchillebeeckx/cherry-mx-switch).

TODO: replace with the footprints and models found [here](https://github.com/kiswitch/kiswitch).

## 3D Printed Parts
TODO

## Microcontroller Code
Programmed with VSCode for the RP2040. See [mcu](mcu/) directory for the files.

## Desktop Code
There exist two versions to target Windows and Linux respectively. Both connect to the JukeBox over USB serial to control RGB and show fun graphics on the screen. Eventually, it'd be nice to unify the two under a single project.

# License
Copyright (c) 2020-2023 Logan "NotQuiteApex" Hickok-Dickson

This project has two discrete parts, the CAD files and the programming files. Programming files, generally found in the MCU folder, are licensed under the MIT license. All CAD files, generally found in the Board folder, (unless provided by an external source, such as PCB footprints and STEP models) are licensed under [CC BY-SA-NC](https://creativecommons.org/licenses/by-nc-sa/4.0/).

If you would like to sell a variation of the board designed by you, reach out and an alternative license can be discussed and granted.
