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

    uint16_t sx = x, sy = y;

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
                    uint16_t tx, ty;
                    tx = ((font_width-c) + i * (kern_width-2) + x - sx) * s + sx;
                    ty = (r + y - sy) * s + sy;
                    for (uint8_t by=0; by<s; by++) {
                        for (uint8_t bx=0; bx<s; bx++) {
                            lcd_put(tx + bx, ty + by);
                        }
                    }
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

    if (screenstate == WaitingConnection) {
        lcd_print_raw("JukeBoxStats", 20, 114, 2);
        lcd_print_raw("Waiting for connection...", 15, 156, 1);
    } else if (screenstate == ShowStats) {
        if (strncmp(cpuName, "AMD", 3) == 0) {
            lcd_set_color(255, 63, 0);
        } else if (strncmp(cpuName, "INTEL", 5) == 0) {
            lcd_set_color(0, 127, 255);
        } else {
            lcd_set_color(255, 0, 255);
        }
        lcd_print_raw(cpuName, 0, 0, 1);
        lcd_set_color(255, 255, 255);

        lcd_print_raw(cpuFreq,   0, 14, 2);
        lcd_print_raw(cpuLoad,  88, 14, 2);
        lcd_print_raw(cpuTemp, 170, 14, 2);

        lcd_print_raw("FreqGHz",       0, 38, 1);
        lcd_print_raw("Load%",        88, 38, 1);
        lcd_print_raw("Temp\xF8""C", 170, 38, 1);

        if (strncmp(gpuName, "AMD", 3) == 0) {
            lcd_set_color(255, 0, 0);
        } else if (strncmp(gpuName, "NVIDIA", 6) == 0) {
            lcd_set_color(127, 255, 127);
        } else {
            lcd_set_color(255, 0, 255);
        }
        lcd_print_raw(gpuName, 0, 80, 1);
        lcd_set_color(255, 255, 255);
        
        lcd_print_raw(gpuCoreLoad,   0, 94, 2);
        lcd_print_raw(gpuVramLoad,  88, 94, 2);
        lcd_print_raw(gpuTemp,     170, 94, 2);

        lcd_print_raw("Load%",         0, 118, 1);
        lcd_print_raw("Vram%",        88, 118, 1);
        lcd_print_raw("Temp\xF8""C", 170, 118, 1);

        lcd_print_raw(gpuCoreClock,  0, 132, 2);
        lcd_print_raw(gpuVramClock, 138, 132, 2);

        lcd_print_raw("CoreMHz",   0, 156, 1);
        lcd_print_raw("VramMHz", 138, 156, 1);

        lcd_set_color(255,   0, 255);
        lcd_print_raw("RAM:",     0, 196, 1);
        lcd_set_color(255, 255, 255);
        lcd_print_raw(ramUsed,   60, 196, 2);
        lcd_print_raw("/",      130, 206, 1);
        lcd_print_raw(ramCount, 144, 206, 1);
    }

    lcd_present();
}
