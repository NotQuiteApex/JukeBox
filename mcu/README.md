# Code for the JukeBox
This code was written specifically for the ATmega8U2/16U2 microcontroller unit.

## udev Rule
For Linux computers, it may be necessary to change the permissions of a DFU USB device to allow writing to the device for a non-root user. The udev rules included in misc will automatically set the permissions correctly. They may be placed in `/etc/udev/rules.d/` to be used.

## Resources Used
- [LUFA Project](https://github.com/abcminiuser/lufa)
- [LUFA Keyboard Demo](https://github.com/abcminiuser/lufa/tree/master/Demos/Device/ClassDriver/Keyboard)
- [LUFA for PlatformIO](https://github.com/sanjay900/pio-bare-lufa)
- [udev rule for ATmega8U2 DFU](https://github.com/samhocevar-forks/qmk-firmware/blob/master/docs/faq_build.md)

# License
Copyright (c) 2020-2022 Logan Hickok-Dickson

All code in this repository is licensed under MIT, not including external libraries used such as LUFA.
