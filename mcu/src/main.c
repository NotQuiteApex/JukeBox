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

int main() {
    setup_default_uart();
    printf("Hello, world!\n");

    #ifdef PICO_DEFAULT_LED_PIN
        gpio_init(LED_PIN);
        gpio_set_dir(LED_PIN, GPIO_OUT);
    #endif

    // setup keyboard pins
    for (uint8_t i=0; i<KB_COL_COUNT; i++) {
        gpio_init(KB_COL + i);
        gpio_set_dir(KB_COL + i, GPIO_IN);
        gpio_pull_down(KB_COL + i);
    }
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

static void send_hid_report(uint8_t report_id, uint32_t btn) {
  // skip if hid is not ready yet
    if ( !tud_hid_ready() )
        return;

    switch(report_id)
    {
        case REPORT_ID_KEYBOARD: {
            // use to avoid send multiple consecutive zero report for keyboard
            static bool has_keyboard_key = false;

            if ( btn ) {
                uint8_t keycode[6] = { 0 };
                keycode[0] = HID_KEY_F13;
                keycode[1] = HID_KEY_F14;
                keycode[2] = HID_KEY_F15;
                keycode[3] = HID_KEY_F16;
                keycode[4] = HID_KEY_F17;
                keycode[5] = HID_KEY_F18;
                tud_hid_keyboard_report(REPORT_ID_KEYBOARD, 0, keycode);
                has_keyboard_key = true;
            } else {
                // send empty key report if previously has key pressed
                if (has_keyboard_key) {
                    tud_hid_keyboard_report(REPORT_ID_KEYBOARD, 0, NULL);
                }
                has_keyboard_key = false;
            }
        }
        
        default: break;
    }
}

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

    uint32_t const btn = !gpio_get(KB_COL); //board_button_read();

    // Remote wakeup
    if ( tud_suspended() && btn ) {
        // Wake up host if we are in suspend mode
        // and REMOTE_WAKEUP feature is enabled by host
        tud_remote_wakeup();
    } else {
        // Send the 1st of report chain, the rest will be sent by tud_hid_report_complete_cb()
        send_hid_report(REPORT_ID_KEYBOARD, btn);
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

    uint8_t next_report_id = report[0] + 1;

    // if (next_report_id < REPORT_ID_COUNT) {
    //     send_hid_report(next_report_id, board_button_read());
    // }
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
    // TODO not Implemented
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
