# Programming the Microcontroller From Scratch (On Windows)
This is to document how I got to programming the ATMEGA8U2 on the JukeBox, because this was a tedious, not well documented process due to lack of instructions on how to install the drivers. Presumably there is no such issue on MacOS or Linux.

## 0. Getting started
You will need:
- [FLIP from ATMEL](https://www.microchip.com/en-us/development-tool/flip)
- A USB Connection to the microcontroller.
- Admin privleges on Windows.

## 1. Installing FLIP
It is preferable you download the version that INCLUDES the Java Runtime Environment for easy setup. Download it from the link above.

## 2. Plug in your device, it should be in DFU mode and display as such
Check device manager, it should display the new ATMEGA8U2 as a device with missing drivers.

## 3. Install the USB drivers
These drivers are only for programming a device in DFU, they are not needed for the end product JukeBox. Install the drivers by right clicking on the chip in Device Manager, then Update Driver, then select local/on-your-computer drivers. Then select the folder the drivers are located in, this is usually `C:\Program Files (x86)\Atmel\Flip 3.4.7\usb` if you installed it to the default location.

## 4. Selecting the device for programming
Open FLIP. Select the microchip icon on the top far left, and select ATMEGA8U2. Then select the USB icon next to the first icon, and click the "Open" button. The rest of the icons in FLIP should light up, signalling that the chip is ready to program.
