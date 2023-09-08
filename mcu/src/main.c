// JukeBox V5 Firmware

#include <stdio.h>
#include "pico/stdlib.h"

#include "bsp/board.h"
#include "tusb.h"

#include "usb_descriptors.h"

#ifndef PICO_DEFAULT_LED_PIN
#warning blink example requires a board with a regular LED
#else
    const uint LED_PIN = PICO_DEFAULT_LED_PIN;
#endif

const uint KB_COL = 2;
const uint KB_COL_COUNT = 4;
const uint KB_ROW = 2;
const uint KB_ROW_COUNT = 4;

// https://github.com/ArmDeveloperEcosystem/st7789-library-for-pico
const uint SCR_DIN = 2;
const uint SCR_CLK = 2;
const uint SCR_CS  = 2; // Move pin to ground?
const uint SCR_DC  = 2;
const uint SCR_RST = 2;
const uint SCR_BL  = 2; // Control with GPIO

void blinking_task(void) {
    static uint16_t blink_interval_ms = 1000;
    static uint64_t start_ms = 0;
    static bool led_state = false;

    // Blink every interval ms
    if ( time_us_64() / 1000 - start_ms < blink_interval_ms)
        return; // not enough time
    start_ms += blink_interval_ms;
    
    led_state = !led_state;
    #ifdef PICO_DEFAULT_LED_PIN
        gpio_put(LED_PIN, led_state ? 1 : 0);
    #endif
}

void hid_task(void);

int main() {
    // setup_default_uart();
    // printf("Hello, world!\n");

    #ifdef PICO_DEFAULT_LED_PIN
        gpio_init(LED_PIN);
        gpio_set_dir(LED_PIN, GPIO_OUT);
    #endif

    // setup keyboard pins
    // column pins are pulldown inputs. if their current state is high, then the key is down.
    for (uint8_t i=0; i<KB_COL_COUNT; i++) {
        gpio_init(KB_COL + i);
        gpio_set_dir(KB_COL + i, GPIO_IN);
        gpio_pull_down(KB_COL + i);
    }
    // row pins are outputs. they produce current to check each row of keys.
    for (uint8_t i=0; i<KB_ROW_COUNT; i++) {
        gpio_init(KB_ROW + i);
        gpio_set_dir(KB_ROW + i, GPIO_OUT);
        gpio_put(KB_ROW + i, 0);
    }

    tusb_init();

    while (true) {
        tud_task();

        blinking_task();

        hid_task();
    }

    return 0;
}


//--------------------------------------------------------------------+
// Device callbacks
//--------------------------------------------------------------------+

// Invoked when device is mounted
void tud_mount_cb(void) {}

// Invoked when device is unmounted
void tud_umount_cb(void) {}

// Invoked when usb bus is suspended
// remote_wakeup_en : if host allow us  to perform remote wakeup
// Within 7ms, device must draw an average of current less than 2.5 mA from bus
void tud_suspend_cb(bool remote_wakeup_en) {}

// Invoked when usb bus is resumed
void tud_resume_cb(void) {}


//--------------------------------------------------------------------+
// USB HID
//--------------------------------------------------------------------+

// Every 10ms, we will sent 1 report for each HID profile (keyboard, mouse etc ..)
// tud_hid_report_complete_cb() is used to send the next report after previous one is complete
void hid_task(void) {
    // Poll every 10ms
    const uint32_t interval_ms = 10;
    static uint64_t start_ms = 0;

    if ( time_us_64() / 1000 - start_ms < interval_ms) {
        return; // not enough time
    }
    start_ms += interval_ms;

    static uint8_t keys [] = {
		HID_KEY_F13, HID_KEY_F14, HID_KEY_F15, HID_KEY_F16,
		HID_KEY_F17, HID_KEY_F18, HID_KEY_F19, HID_KEY_F20,
		HID_KEY_F21, HID_KEY_F22, HID_KEY_F23, HID_KEY_F24,
    };

    uint8_t k = 0;
    uint8_t keycodes[6] = {0};
    uint8_t usedKeys = 0;
    for (uint8_t row=0; row<KB_ROW_COUNT; row++) {
        gpio_put(KB_ROW + row, 1);
        for (uint8_t col=0; col<KB_COL_COUNT; col++) {
            if (usedKeys >= 6) {
                continue;
            }

            if (gpio_get(KB_COL + col)) {
                keycodes[usedKeys++] = keys[k];
            }

            k++;
        }
        gpio_put(KB_ROW + row, 0);
    }

    if (tud_suspended() && usedKeys > 0) {
        tud_remote_wakeup();
    } else if (!tud_hid_ready()) {
        tud_hid_keyboard_report(REPORT_ID_KEYBOARD, 0, keycodes);
    }
}

// Invoked when sent REPORT successfully to host
// Application can use this to send the next report
// Note: For composite reports, report[0] is report ID
void tud_hid_report_complete_cb(
    uint8_t instance,
    uint8_t const* report,
    uint16_t len
) {
    (void) instance;
    (void) len;
}

// Invoked when received GET_REPORT control request
// Application must fill buffer report's content and return its length.
// Return zero will cause the stack to STALL request
uint16_t tud_hid_get_report_cb(
    uint8_t instance,
    uint8_t report_id,
    hid_report_type_t report_type,
    uint8_t* buffer,
    uint16_t reqlen
) {
    (void) instance;
    (void) report_id;
    (void) report_type;
    (void) buffer;
    (void) reqlen;

    return 0;
}

// Invoked when received SET_REPORT control request or
// received data on OUT endpoint ( Report ID = 0, Type = 0 )
void tud_hid_set_report_cb(
    uint8_t instance,
    uint8_t report_id,
    hid_report_type_t report_type,
    uint8_t const* buffer,
    uint16_t bufsize
) {
    (void) instance;

    if (report_type == HID_REPORT_TYPE_OUTPUT) {
        // Set keyboard LED e.g Capslock, Numlock etc...
        if (report_id == REPORT_ID_KEYBOARD) {
            // bufsize should be (at least) 1
            if ( bufsize < 1 ) return;

            uint8_t const kbd_leds = buffer[0];

            // if (kbd_leds & KEYBOARD_LED_CAPSLOCK) {
            //     // Capslock On: disable blink, turn led on
            //     blink_interval_ms = 0;
            //     board_led_write(true);
            // } else {
            //     // Caplocks Off: back to normal blink
            //     board_led_write(false);
            //     blink_interval_ms = BLINK_MOUNTED;
            // }
        }
    }
}