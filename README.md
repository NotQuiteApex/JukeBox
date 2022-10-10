# JukeBox Board
A board that exposes the keys F13-24 for use with macros, or hotkeys for streaming (or anything, really).

## Printed Circuit Board
ATmega8u2 (Microcontroller logic), USB4085-GF-A (USB-C Receptacle), AS-16.000-20-EXT (16MHz Crystal oscillator for USB comms). See `board` directory.

Footprints and model for USB4085-GF-A can be found [here](https://www.mouser.com/ProductDetail/GCT/USB4085-GF-A?qs=KUoIvG%2F9Ilba1bQOahfWjw%3D%3D). Footprints for Kailh Choc's can be found [here](https://github.com/daprice/keyswitches.pretty). The model for the Cherry MX Switch can be found [here](https://github.com/ConstantinoSchillebeeckx/cherry-mx-switch).

## 3D Printed Parts
Created with OpenSCAD. See `prints` directory.

## Microcontroller Code
Programmed with PlatformIO with VSCode. See `mcu` directory.

# License
Copyright (c) 2020-2022 Logan Hickok-Dickson

This project has two discrete parts, the CAD files and the programming files. Programming files, generally found in the MCU folder, are licensed under the MIT license. All CAD files, generally found in the Board folder, (unless provided by an external source, such as PCB footprints and STEP models) are licensed under [CC BY-SA-NC](https://creativecommons.org/licenses/by-nc-sa/4.0/).

If you would like to sell a variation of the board designed by you, reach out and an alternative license can be discussed and granted.
