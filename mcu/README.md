# JukeBox V5 Firmware
Made for the RPi Pico.

Dependencies:
```
sudo apt install cmake gcc-arm-none-eabi libnewlib-arm-none-eabi libstdc++-arm-none-eabi-newlib
```

To build (NOTE: You may need to initialize git submodules):
```
mkdir build
cd build
cmake ..
make
```

# License
Copyright (c) 2020-2023 Logan Hickok-Dickson

All code in this repository is licensed under MIT, not including external libraries used such as the Pico SDK.