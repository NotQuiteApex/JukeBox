# JukeBox PCB
Made with KiCad 8, the PCB is the physical heart of the project.

Footprints and models:
- [Cherry/Kailh Switch footprint based on this.](https://github.com/luke-schutt/Pi5Keyboard/blob/main/Pi5-pcb/Pi5Footprints.pretty/Low%20Profile%20GC%20plus%20MX.kicad_mod)

Estimated power usage is 5 volts at 0.1 amps, or 0.5 watts.

## Bill of Materials
| Ref         | Part No.           | Qty | Value |
|-------------|--------------------|-----|-------|
| R1, R5, R14 | RMCF0402FT1K00     | 3   | 1k    |
| R3, R4      | RMCF0402FT27R0     | 2   | 27    |
| R6, R7      | RMCF0402FT5K11     | 2   | 5.11k |
| R8-12, R15  | RMCF0402FT10K0     | 5   | 10k   |
| R13         | RMCF0402FT10R0     | 1   | 10    |
| C1, C4      | GRM21BR61C106KE15K | 2   | 10u   |
| C2, C3      | GCM1555C1H150JA16D | 2   | 15p   |
| C5          | GRM155R71E104KE14J | 1   | 100n  |
| C8, C10     | GRM155C81A105KA12D | 2   | 1u    |
| D1-12       | 1N4148W-SOD-123    | 12  |       |
| D13-24      | WS2812B-2020       | 12  |       |
| D25         | 150080GS75000      | 1   |       |
| J1          | USB4105-GF-A       | 1   |       |
| Q1          | S8050-SOT-23       | 1   |       |
| U1          | AZ1117IH-3.3TRG1   | 1   |       |
| U2          | W25Q128JVSIQ-TR    | 1   |       |
| U3          | RP2040             | 1   |       |
| U4          | TFTQ-T20SH12ZP01   | 1   |       |
| U5          | CAT24C512          | 1   |       |
| Y1          | ABM8-272-T3        | 1   |       |
