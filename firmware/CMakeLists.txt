cmake_minimum_required(VERSION 3.13)

# initialize pico-sdk from submodule
# note: this must happen before project()
include(pico-sdk/pico_sdk_init.cmake)

enable_language( C CXX ASM )

# Name the project
project(jukebox_v5)

# initialize the Raspberry Pi Pico SDK
pico_sdk_init()

# Source files
add_executable(jukebox_firmware
    "src/main.c"

    "src/keyboard.c"
    "src/lcd.c"
    "src/led.c"
    "src/rgb.c"
    "src/serial.c"

    "src/callbacks.c"

    "src/st7789_lcd.c"
    
    "src/usb_descriptors.c"
)

# PIO files
pico_generate_pio_header(jukebox_firmware ${CMAKE_CURRENT_LIST_DIR}/src/st7789_lcd.pio)
pico_generate_pio_header(jukebox_firmware ${CMAKE_CURRENT_LIST_DIR}/src/ws2812_rgb.pio)

# Header files
target_include_directories(jukebox_firmware PUBLIC
    "include/"
)

# Libraries
target_link_libraries(jukebox_firmware PUBLIC
    pico_stdlib
    pico_rand
    pico_unique_id
    pico_multicore
    hardware_pio
    tinyusb_device
    tinyusb_board
)

# Create map/bin/hex/uf2 file in addition to ELF.
pico_add_extra_outputs(jukebox_firmware)
