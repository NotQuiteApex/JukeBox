# JukeBox V5
A board that exposes the keys F13-F24 for use with macros, or hotkeys for streaming (or anything, really)!

TODO: Pictures and badges here.

### Compatible programs
Sadly, not every program is compatible with these additional function keys, and some do not allow the use of these keys. There is a running list [here](docs/program-compatibility.md) of what programs do and do not work with the keys provided by JukeBox.

# Firmware
Designed for the RP2040 chip, by Raspberry Pi. Tested using an RPi Pico. See [`firmware`](firmware/) directory for the files.

### Build and Flash
Package manager dependencies, to build the firmware (on Debian/Ubuntu):
```
sudo apt install cmake gcc-arm-none-eabi libnewlib-arm-none-eabi libstdc++-arm-none-eabi-newlib
```
These are the only packages necessary for building RP2040 firmware. For more information, visit the [pico-sdk repository](https://github.com/raspberrypi/pico-sdk#quick-start-your-own-project). Equivalent packages exist for most other package managers.

To build the firmware, run the following in the [`firmware`](firmware/) directory:
```bash
git submodule update --init --recursive
mkdir build
cd build
cmake ..
make -j4
```
This updates the pico-sdk for the project, along with all of its dependencies. Then, the build is properly started by setting up the build environment, and finally running make to begin the build process with 4 processing cores. You can increase this number as necessary. The final firmware binary will be located at [`firmware/build/jukebox_firmware.uf2`](firmware/build/jukebox_firmware.uf2), move this file onto the board when it's in programming mode (held BOOTSEL) to flash the new firmware.

### Screen Font
The default font is included as [`firmware/misc/JukeBoxFont.png`](firmware/misc/JukeBoxFont.png). It is a monospace, 12x12 pixel font, with the sheet matching IBM Code Page 437. It's ASCII compatible, but with plenty of extra symbols. To export it, run the included [`mkfont.py`](firmware/misc/mkfont.py) script to generate a new `font.h` file. Then, before building the frimware, replace the existing `font.h` with your newly generated font.

# Hardware

### Printed Circuit Board
Made with KiCad 6.0.7, the PCB is the physical heart of the project. See [`hardware/board`](hardware/board/) directory for the files. You can access the BOM [here](https://www.digikey.com/en/mylists/list/QC5ACPN0J3).

Footprints and model for USB4105-GF-A can be found [here](https://www.digikey.com/en/products/detail/gct/usb4105-gf-a/11198441). Footprints for Kailh Choc's can be found [here](https://github.com/daprice/keyswitches.pretty), model can be found [here](https://grabcad.com/library/kailh-low-profile-mechanical-keyboard-switch-1). The model for the Cherry MX Switch can be found [here](https://github.com/ConstantinoSchillebeeckx/cherry-mx-switch).

TODO: replace with the footprints and models found [here](https://github.com/kiswitch/kiswitch).

### 3D Printed Parts
TODO: put prints in [`hardware/prints`](hardware/prints/) directory.

# Software
The desktop app that connects to the JukeBox to control its RGB and display, written in Rust for Windows and Linux. See [`software`](software/) directory for the files.

TODO: add gpu support to Rust version through nvml-wrapper crate, AMD Display Library through Rust wrappers, and Intel Graphics Control Library through Rust wrappers.

# License
Copyright (c) 2020-2024 Logan "NotQuiteApex" Hickok-Dickson

This project has two discrete parts, the programming files and the CAD files. Programming files found in the [`firmware`](firmware/) and [`software`](software/) folders, are licensed under the MIT license. All CAD files found in the [`hardware`](software/) folder, (unless provided by an external source, such as PCB footprints and STEP models) are licensed under [CC BY-NC-SA](https://creativecommons.org/licenses/by-nc-sa/4.0/).

If you would like to sell a variation of the board designed by you, reach out and an alternative license can be discussed and granted.
