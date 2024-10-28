# JukeBox PCB
Made with KiCad 8, the PCB is the physical heart of the project.

Footprints and models:
- [USB4105-GF-A](https://www.digikey.com/en/products/detail/gct/usb4105-gf-a/11198441)
- [W25Q16JVUXIQ TR](https://www.digikey.com/en/products/detail/winbond-electronics/W25Q16JVUXIQ-TR/15182017)
- [Cherry/Kailh Switch footprint based on this.](https://github.com/luke-schutt/Pi5Keyboard/blob/main/Pi5-pcb/Pi5Footprints.pretty/Low%20Profile%20GC%20plus%20MX.kicad_mod)

Estimated power usage is 5 volts at 0.1 amps, or 0.5 watts.

## Bill of Materials
| Ref     | Part No.             | Qty | Value | SKU |
|---------|----------------------|-----|-------|-----|
| C1, C4  | CL21A106KOQNNNE      | 2   | 10u   | 0   |
| C2, C3  | CL21C270JCANNNC      | 2   | 27p   | 0   |
| C5      | CL21B104KACNNNC      | 1   | 100n  | 0   |
| C8, C10 | CL21B105KAFNNNG      | 2   | 1u    | 0   |
| D1-12   | 1N4148W-SOD-123      | 12  |       | 0   |
| D13-24  | WS2812B-2020         | 12  |       | 1/3 |
| D25     | 150080GS75000        | 1   |       | 0   |
| J1      | USB4105-GF-A         | 1   |       | 0   |
| J2      | JST-SH-SM04B-SRSS-TB | 1   |       | 0   |
| Q1      | S8050-SOT-23         | 1   |       | 2/3 |
| R1, R5  | RMCF0805FT1K00       | 2   | 1k    | 0   |
| R3, R4  | RMCF0805JT27R0       | 2   | 27    | 0   |
| R6, R7  | RNCP0805FTD5K11      | 2   | 5.11k | 0   |
| R8-11   | RNCP0805FTD10K0      | 4   | 10k   | 0   |
| R12     | RMCF0805JT600R       | 1   | 600   | 0   |
| R13     | RMCF0805JT10R0       | 1   | 10    | 2/3 |
| R14     | RMCF0805FT1K00       | 1   | 1k    | 2/3 |
| R15     | RNCP0805FTD10K0      | 1   | 10k   | 2/3 |
| U1      | AZ1117IH-3.3TRG1     | 1   |       | 0   |
| U2      | W25Q16JVUXIQ TR      | 1   |       | 0   |
| U3      | RP2040-SC0914(13)    | 1   |       | 0   |
| U4      | TFTQ-T20SH12ZP01     | 1   |       | 2/3 |
| Y1      | ECS-120-20-33-AEL-TR | 1   |       | 0   |

## SKUs
0. Default
1. Default + RGB
2. Default + Screen
3. Default + RGB + Screen
