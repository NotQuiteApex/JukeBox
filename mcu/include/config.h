// File for controlling various aspects of of the JukeBox firmware.
// If disabled in this file, certain modules will not work without reflashing the firmware.

#ifndef JUKEBOX_CONFIG_H
#define JUKEBOX_CONFIG_H

// -----------------------------------------------------------------------------
// Module Switches
// -----------------------------------------------------------------------------

// Screen, for displaying things like PC stats.
#define JB_MOD_SCREEN
// RGB LEDs, for bright fun colors to entertain the children.
#define JB_MOD_RGBLEDS


// -----------------------------------------------------------------------------
// Module Config
// -----------------------------------------------------------------------------

// Human Input Device
#define JB_HID_REFRESH_INTERVAL 10
#define JB_HID_REFRESH_OFFSET 0

#define JB_HID_KB_COL     19
#define JB_HID_KB_COL_NUM 4
#define JB_HID_KB_ROW     16
#define JB_HID_KB_ROW_NUM 3

// Serial (for screen)
#define JB_SERIAL_REFRESH_INTERVAL 250
#define JB_SERIAL_REFRESH_OFFSET 0

// Screen
#ifdef JB_MOD_SCREEN
    #define JB_SCREEN_REFRESH_INTERVAL 250
    #define JB_SCREEN_REFRESH_OFFSET 100

    #define JB_SCREEN_DRAW_INTERVAL 250
    #define JB_SCREEN_DRAW_OFFSET 200

    #define JB_SCREEN_PIN_DIN 0
    #define JB_SCREEN_PIN_CLK 1
    #define JB_SCREEN_PIN_CS  2
    #define JB_SCREEN_PIN_DC  3
    #define JB_SCREEN_PIN_RST 4
    #define JB_SCREEN_PIN_BL  5

    #define JB_SCREEN_CLK_DIV 1.f

    #define JB_SCREEN_RESOLUTION_WIDTH 240
    #define JB_SCREEN_RESOLUTION_HEIGHT 320

    #define JB_SCREEN_ORIENTATION JB_PORTRAIT
    #define JB_SCREEN_MIRROR_FLIP
#endif

// RGB LEDs
#ifdef JB_MOD_RGBLEDS
    #define JB_RGBLEDS_REFRESH_INTERVAL 250
    #define JB_RGBLEDS_REFRESH_OFFSET 100

    #define JB_RGBLEDS_PIN 6
    #define JB_RGBLEDS_FREQ 800000.f
    // #define JB_RGBLEDS_IS_RGBW
#endif

#endif // JUKEBOX_CONFIG_H
