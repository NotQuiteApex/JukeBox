cmake_minimum_required(VERSION 3.13)

# initialize pico-sdk from submodule
# note: this must happen before project()
include(pico-sdk/pico_sdk_init.cmake)

project(jukebox_v5)

# initialize the Raspberry Pi Pico SDK
pico_sdk_init()

# rest of your project
add_executable(jukebox_firmware
    src/main.c
)

# Add pico_stdlib library which aggregates commonly used features
target_link_libraries(jukebox_firmware
    pico_stdlib
    tinyusb_device
)

# create map/bin/hex/uf2 file in addition to ELF.
pico_add_extra_outputs(jukebox_firmware)
