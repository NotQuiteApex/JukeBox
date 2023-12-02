#include "lcd.h"

#include <stdarg.h>
#include <string.h>

#include "font.h"
#include "st7789_lcd.h"
#include "serial.h"


ScreenState screenstate, previousstate;
extern uint32_t countermax;
extern SerialStage commstage;
extern int commstagepart;


inline uint16_t lcd_rgb565(uint8_t r, uint8_t g, uint8_t b) {
    // https://stackoverflow.com/a/76442697/13977827
    return ((r>>3) << 11) | ((g>>2) << 5) | b >> 3;
}

uint16_t lcd_color_full = 65535;
// uint8_t lcd_color_r = 255;
// uint8_t lcd_color_g = 255;
// uint8_t lcd_color_b = 255;

inline void lcd_set_color(uint8_t r, uint8_t g, uint8_t b) {
    // lcd_color_r = r;
    // lcd_color_g = g;
    // lcd_color_b = b;
    lcd_color_full = lcd_rgb565(r, g, b);
}

inline void lcd_clear(void) {
    st7789_fb_clear();
}

inline void lcd_present(void) {
    st7789_lcd_push_fb();
}

inline void lcd_put(uint16_t x, uint16_t y) {
    st7789_fb_put(lcd_color_full, x, y);
}

inline void lcd_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h) {
    for (uint16_t rh=0; rh<h; rh++) {
        for (uint16_t rw=0; rw<w; rw++) {
            lcd_put(x + rw, y + rh);
        }
    }
}

void lcd_print(char * text, uint16_t x, uint16_t y, uint8_t s) {

}

void lcd_print_raw(char * text, uint16_t x, uint16_t y, uint8_t s) {
    // TODO: use `s` for scaling text

    uint16_t len = strlen(text);
    for (uint16_t i=0; i<len; i++) {
        char t = text[i];
        for (uint8_t r=0; r<font_height; r++) {
            uint16_t row = font[t][r];
            if (row == 0) {
                continue;
            }
            for (uint8_t c=0; c<font_width; c++) {
                if (row & BIT(c)) {
                    lcd_put((font_width-c) + i * (font_width-2) + x, r + y);
                }
            }
        }
    }
}

// TODO: make an sprintf like function.
// void sfmt(char * buf, char * fmt, ...) {
//     va_list args;
//     va_start(args, fmt);

//     while (*fmt != '\0') {

//     }

//     va_end(args);
// }

inline void lcd_init(void) {
    st7789_lcd_init();
    
    previousstate = Unknown;
    screenstate = WaitingConnection;
}

void lcd_task(void) {
    REFRESH_CHECK(JB_SCREEN_REFRESH_INTERVAL, JB_SCREEN_REFRESH_OFFSET);
    
    lcd_clear();

    const int scr_scale = 1; // temp

    if (screenstate == WaitingConnection) {
        lcd_print_raw("JukeBoxStats", 55, 50, scr_scale);
        lcd_print_raw("Waiting for connection...", 5, 60, scr_scale);
    } else if (screenstate == ShowStats) {
        if (strncmp(cpuName, "AMD", 3) == 0) {
            lcd_set_color(255, 63, 0);
        } else {
            lcd_set_color(0, 127, 255);
        }
        lcd_print_raw(cpuName, 0, 0, scr_scale);
        lcd_set_color(255, 255, 255);

        lcd_print_raw(cpuFreq,   0, 14, 2);
        lcd_print_raw(cpuLoad,  88, 14, 2);
        lcd_print_raw(cpuTemp, 152, 14, 2);

        lcd_print_raw("FreqGHz",       0, 28, scr_scale);
        lcd_print_raw("Load%",        88, 28, scr_scale);
        lcd_print_raw("Temp\xF8""C", 152, 28, scr_scale);

        if (strncmp(gpuName, "AMD", 3) == 0) {
            lcd_set_color(255, 0, 0);
        } else {
            lcd_set_color(127, 255, 127);
        }
        lcd_print_raw(gpuName, 0, 64, scr_scale);
        lcd_set_color(255, 255, 255);
        
        lcd_print_raw(gpuCoreLoad,   0, 78, 2);
        lcd_print_raw(gpuVramLoad,  88, 78, 2);
        lcd_print_raw(gpuTemp,     150, 78, 2);

        lcd_print_raw("Load%",         0, 92, scr_scale);
        lcd_print_raw("Vram%",        88, 92, scr_scale);
        lcd_print_raw("Temp\xF8""C", 150, 92, scr_scale);

        lcd_print_raw(gpuCoreClock,  0, 106, scr_scale);
        lcd_print_raw(gpuVramClock, 88, 106, scr_scale);

        lcd_print_raw("CoreMHz",  0, 120, scr_scale);
        lcd_print_raw("VramMHz", 88, 120, scr_scale);

        lcd_set_color(255,   0, 255);
        lcd_print_raw("RAM:",     0, 160, scr_scale);
        lcd_set_color(255, 255, 255);
        lcd_print_raw(ramUsed,   60, 160, scr_scale);
        lcd_print_raw("/",      106, 160, scr_scale);
        lcd_print_raw(ramCount, 120, 160, scr_scale);
    }

    lcd_present();
}
