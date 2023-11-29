// JukeBox V5 Firmware

#include "common.h"

#include "keyboard.h"
#include "lcd.h"
#include "led.h"
#include "rgb.h"
#include "serial.h"


int main() {
    led_init();
    keyboard_init();

    serial_init();

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
        serial_task();

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
