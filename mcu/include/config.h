// File for controlling various aspects of of the JukeBox firmware.
// If disabled in this file, certain modules will not work without reflashing the firmware.

#ifndef JUKEBOX_CONFIG_H
#define JUKEBOX_CONFIG_H

// -----------------------------------------------------------------------------
// Module Switches
// -----------------------------------------------------------------------------

// Screen, for displaying things like PC stats.
#define JUKEBOX_MOD_SCREEN
// RGB LEDs, for bright fun colors to entertain the children.
#define JUKEBOX_MOD_RGBLEDS


// -----------------------------------------------------------------------------
// Module Config
// -----------------------------------------------------------------------------

// Human Input Device
#define JUKEBOX_HID_REFRESH_INTERVAL 50
#define JUKEBOX_HID_REFRESH_OFFSET 0

// Serial (for screen)
#define JUKEBOX_SERIAL_REFRESH_INTERVAL 250
#define JUKEBOX_SERIAL_REFRESH_OFFSET 0

// Screen
#ifdef JUKEBOX_MOD_SCREEN
    #define JUKEBOX_SCREEN_REFRESH_INTERVAL 250
    #define JUKEBOX_SCREEN_REFRESH_OFFSET 125

    #define JUKEBOX_SCREEN_PIN_DIN 0
    #define JUKEBOX_SCREEN_PIN_CLK 1
    #define JUKEBOX_SCREEN_PIN_CS  2
    #define JUKEBOX_SCREEN_PIN_DC  3
    #define JUKEBOX_SCREEN_PIN_RST 4
    #define JUKEBOX_SCREEN_PIN_BL  5

    #define JUKEBOX_SCREEN_SERIAL_CLK_DIV 1.f

    #define JUKEBOX_SCREEN_RESOLUTION_WIDTH 240
    #define JUKEBOX_SCREEN_RESOLUTION_HEIGHT 320

    #define JUKEBOX_SCREEN_ORIENTATION PORTRAIT
    // #define JUKEBOX_SCREEN_MIRROR_FLIP
#endif

// RGB LEDs
#ifdef JUKEBOX_MOD_RGBLEDS


    #define JUKEBOX_RGBLEDS_PIN 6
    #define JUKEBOX_RGBLEDS_FREQ 800000.f
    // #define JUKEBOX_RGBLEDS_RGBW
#endif

#endif // JUKEBOX_CONFIG_H
