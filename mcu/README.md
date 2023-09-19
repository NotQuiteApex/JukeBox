# JukeBox V5 Firmware
Made for the RPi Pico.

Dependencies:
```
sudo apt install cmake gcc-arm-none-eabi libnewlib-arm-none-eabi libstdc++-arm-none-eabi-newlib
```

To build (NOTE: You may need to `git submodule update --init --recursive`):
```
mkdir build
cd build
cmake ..
make
```

# Font
The default font is included as `JukeBoxFont.png`. It is a monospace, 12x12 pixel font, with the sheet matching IBM Code Page 437. It's ASCII compatible, but with plenty of extra symbols. To export it, !!TODO!!

# License
Copyright (c) 2020-2023 Logan Hickok-Dickson

All code in this repository is licensed under MIT, not including external libraries used such as the Pico SDK.