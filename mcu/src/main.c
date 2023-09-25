// JukeBox V5 Firmware

#include "common.h"

#include <pico/stdlib.h>
#include <pico/rand.h>

#include <bsp/board.h>
#include <tusb.h>

#include "keyboard.h"
#include "lcd.h"
#include "led.h"
#include "rgb.h"

#include "usb_descriptors.h"


//--------------------------------------------------------------------+
// USB CDC
//--------------------------------------------------------------------+

void cdc_task(void) {
    REFRESH_CHECK(JB_SERIAL_REFRESH_INTERVAL, JB_SERIAL_REFRESH_OFFSET);

    if (tud_cdc_available()) {
        // read datas
        char buf[64];
        uint32_t count = tud_cdc_read(buf, sizeof(buf));
        (void) count;
    }
}


//--------------------------------------------------------------------+
// Main
//--------------------------------------------------------------------+

int main() {
    led_init();
    keyboard_init();

    #ifdef JB_MOD_SCREEN
        lcd_init();
    #endif
    #ifdef JB_MOD_RGBLEDS
        rgb_init();
    #endif

    tusb_init();

    while (true) {
        tud_task();

        keyboard_task();
        cdc_task();

        #ifdef JB_MOD_SCREEN
            lcd_task();
            lcd_draw_task();
        #endif

        #ifdef JB_MOD_RGBLEDS
            rgb_task();
        #endif

        led_blinking_task();
    }

    return 0;
}
