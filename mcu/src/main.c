// JukeBox V5 Firmware

#include <pico/stdlib.h>
#include <pico/rand.h>

#include <bsp/board.h>
#include <tusb.h>

#include "lcd.h"
#include "led.h"
#include "keyboard.h"

#include "ws2812_rgb.h"

#include "usb_descriptors.h"


//--------------------------------------------------------------------+
// Device callbacks
//--------------------------------------------------------------------+

// Invoked when device is mounted
void tud_mount_cb(void) {
    led_set_mounted();
}

// Invoked when device is unmounted
void tud_umount_cb(void) {
    led_set_unmounted();
}

// Invoked when usb bus is suspended
// remote_wakeup_en : if the host allowed us to perform remote wakeup
// Within 7ms, device must draw an average of current less than 2.5 mA from bus
void tud_suspend_cb(bool remote_wakeup_en) {
    (void) remote_wakeup_en;
    led_set_suspended();
}

// Invoked when usb bus is resumed
void tud_resume_cb(void) {
    led_set_mounted();
}


//--------------------------------------------------------------------+
// USB CDC
//--------------------------------------------------------------------+

void cdc_task(void) {
    const uint32_t interval_ms = 250;
    static uint64_t start_ms = 0;
    if ( time_us_64() / 1000 - start_ms < interval_ms) {
        return;
    }
    start_ms += interval_ms;

    if (tud_cdc_available()) {
        // read datas
        char buf[64];
        uint32_t count = tud_cdc_read(buf, sizeof(buf));
        (void) count;
    }
}


//--------------------------------------------------------------------+
// USB HID
//--------------------------------------------------------------------+

// Every 10ms, we will sent 1 report for each HID profile (keyboard, mouse etc ..)
// tud_hid_report_complete_cb() is used to send the next report after previous one is complete
void hid_task(void) {
    const uint32_t interval_ms = 10;
    static uint64_t start_ms = 0;
    if ( time_us_64() / 1000 - start_ms < interval_ms) {
        return;
    }
    start_ms += interval_ms;

    keyboard_send_hid_report(REPORT_ID_KEYBOARD);
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

    if (next_report_id < REPORT_ID_COUNT) {
        keyboard_send_hid_report(next_report_id);
    }
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


//--------------------------------------------------------------------+
// Screen
//--------------------------------------------------------------------+

void lcd_task(void) {
    const uint32_t interval_ms = 250;
    static uint64_t start_ms = 0;
    if ( time_us_64() / 1000 - start_ms < interval_ms) {
        return;
    }
    start_ms += interval_ms;

    lcd_set_color(255, 0, 0);
    lcd_print_raw("Testing 0123 !", 5, 5, 1);
    lcd_set_color(0, 255, 255);
    lcd_print_raw("\xd\xe \x2 \x3 \x1 \xd\xe", 5, 30, 1);

    lcd_set_color(255, 255, 255);
    lcd_print_raw("Say hello to the new", 25, 80, 1);
    lcd_set_color(255, 170, 70);
    lcd_print_raw("JukeBox V5 by F.T.I.!", 25, 100, 1);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040000);
    rgb_put_pixel(0x000400);
    rgb_put_pixel(0x000004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040400);
    rgb_put_pixel(0x000404);
    rgb_put_pixel(0x040004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040000);
    rgb_put_pixel(0x000400);
    rgb_put_pixel(0x000004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
}

void lcd_draw_task(void) {
    const uint32_t interval_ms = 1000;
    static uint64_t start_ms = 0;
    if ( time_us_64() / 1000 - start_ms - 125 < interval_ms) {
        return;
    }
    start_ms += interval_ms;
    
    lcd_present();
    lcd_clear();
}


//--------------------------------------------------------------------+
// Main
//--------------------------------------------------------------------+

int main() {
    led_init();
    keyboard_init();

    lcd_init();
    rgb_init();

    tusb_init();

    while (true) {
        tud_task();

        hid_task();
        cdc_task();

        lcd_task();
        lcd_draw_task();

        led_blinking_task();
    }

    return 0;
}
