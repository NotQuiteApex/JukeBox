# JukeBox PCB
Made with KiCad 8, the PCB is the physical heart of the project.

Footprints and models:
- [W25Q16JVUXIQ TR](https://www.digikey.com/en/products/detail/winbond-electronics/W25Q16JVUXIQ-TR/15182017)
- [Cherry/Kailh Switch footprint based on this.](https://github.com/luke-schutt/Pi5Keyboard/blob/main/Pi5-pcb/Pi5Footprints.pretty/Low%20Profile%20GC%20plus%20MX.kicad_mod)

Estimated power usage is 5 volts at 0.1 amps, or 0.5 watts.

## Bill of Materials
| Ref      | Part No.             | Qty | Value |
|----------|----------------------|-----|-------|
| C1, C4   | CL21A106KOQNNNE      | 2   | 10u   |
| C2, C3   | CL21C150JBANFNC      | 2   | 15p   |
| C5       | CL21B104KACNNNC      | 1   | 100n  |
| C8, C10  | CL21B105KAFNNNG      | 2   | 1u    |
| D1-12    | 1N4148W-SOD-123      | 12  |       |
| D13-24   | WS2812B-2020         | 12  |       |
| D25      | 150080GS75000        | 1   |       |
| J1       | USB4105-GF-A         | 1   |       |
| J2       | JST-SH-SM04B-SRSS-TB | 1   |       |
| Q1       | S8050-SOT-23         | 1   |       |
| R1, R5   | RMCF0805FT1K00       | 2   | 1k    |
| R3, R4   | RMCF0805JT27R0       | 2   | 27    |
| R6, R7   | RNCP0805FTD5K11      | 2   | 5.11k |
| R8-11    | RNCP0805FTD10K0      | 4   | 10k   |
| R12      | RMCF0805JT600R       | 1   | 600   |
| R13      | RMCF0805JT10R0       | 1   | 10    |
| R14      | RMCF0805FT1K00       | 1   | 1k    |
| R15      | RNCP0805FTD10K0      | 1   | 10k   |
| R21, R22 | RNCP0805FTD02K0      | 2   | 2k    |
| U1       | AZ1117IH-3.3TRG1     | 1   |       |
| U2       | W25Q64JVSSIQ-TR      | 1   |       |
| U3       | RP2040-SC0914(13)    | 1   |       |
| U4       | TFTQ-T20SH12ZP01     | 1   |       |
| U5       | CAT24C512            | 1   |       |
| Y1       | ABM8-272-T3          | 1   |       |
