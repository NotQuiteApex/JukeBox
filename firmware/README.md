# JukeBox Firmware
Designed for the RP2040 chip, by Raspberry Pi. Tested using an RPi Pico.

### Build and Flash
Package manager dependencies, to build the firmware (on Debian/Ubuntu):
```
sudo apt install cmake gcc-arm-none-eabi libnewlib-arm-none-eabi libstdc++-arm-none-eabi-newlib
```
These are the only packages necessary for building RP2040 firmware. For more information, visit the [pico-sdk repository](https://github.com/raspberrypi/pico-sdk#quick-start-your-own-project). Equivalent packages exist for most other package managers.

To build the firmware, run the following in the current directory:
```bash
git submodule update --init --recursive
mkdir build
cd build
cmake ..
make -j4
```
This updates the pico-sdk for the project, along with all of its dependencies. Then, the build is properly started by setting up the build environment, and finally running make to begin the build process with 4 processing cores. You can increase this number as necessary. The final firmware binary will be located at [`build/jukebox_firmware.uf2`](build/jukebox_firmware.uf2), move this file onto the board when it's in programming mode (held BOOTSEL) to flash the new firmware.

TODO: update above

### Screen Font
The default font is included as [`misc/JukeBoxFont.png`](misc/JukeBoxFont.png). It is a monospace, 12x12 pixel font, with the sheet matching IBM Code Page 437. It's ASCII compatible, but with plenty of extra symbols. To export it, run the included [`misc/mkfont.py`](misc/mkfont.py) script to generate a new `font.h` file. Then, before building the frimware, replace the existing `font.h` with your newly generated font.
