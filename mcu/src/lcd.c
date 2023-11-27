#include "lcd.h"

#include <stdarg.h>
#include <string.h>

#include "font.h"
#include "st7789_lcd.h"
#include "serial.h"


ScreenState screenstate, previousstate;

inline uint16_t lcd_rgb565(uint8_t r, uint8_t g, uint8_t b) {
    // https://stackoverflow.com/a/76442697/13977827
    return ((r>>3) << 11) | ((g>>2) << 5) | b >> 3;
}

uint8_t lcd_color_r = 255;
uint8_t lcd_color_g = 255;
uint8_t lcd_color_b = 255;

inline void lcd_set_color(uint8_t r, uint8_t g, uint8_t b) {
    lcd_color_r = r;
    lcd_color_g = g;
    lcd_color_b = b;
}

inline void lcd_clear(void) {
    st7789_fb_clear();
}

inline void lcd_present(void) {
    st7789_lcd_push_fb();
}

inline void lcd_put(uint16_t x, uint16_t y) {
    st7789_fb_put(lcd_rgb565(lcd_color_r, lcd_color_g, lcd_color_b), x, y);
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
void sfmt(char * buf, char * fmt, ...) {
    va_list args;
    va_start(args, fmt);

    while (*fmt != '\0') {

    }

    va_end(args);
}

inline void lcd_init(void) {
    st7789_lcd_init();
    
    previousstate = Unknown;
    screenstate = WaitingConnection;
}

void lcd_task(void) {
    REFRESH_CHECK(JB_SCREEN_REFRESH_INTERVAL, JB_SCREEN_REFRESH_OFFSET);

    const int scr_scale = 1; // temp
    
    // lcd_print_raw("test", 20, 20, 1);
    // lcd_print_raw(recv, 1, 1, 1);
    
    // char c[2] = {(char) strnlen(recv, 64), 0};
    // lcd_print_raw(c, 1, 44, 1);

    // Drawing on screen!
    if (screenstate != previousstate) {
        previousstate = screenstate;
        // tft.fillScreen(ST77XX_BLACK);
        // tft.setTextSize(1);

        // Initialize the new screen
        if (screenstate == WaitingConnection) {
            lcd_print_raw("MaxStats", 55 * scr_scale, 50 * scr_scale, scr_scale);
            lcd_print_raw("Waiting for connection...", 5 * scr_scale, 60 * scr_scale, scr_scale);
        } else {
            if (strncmp(cpuName, "AMD", 3) == 0) {
                lcd_set_color(255, 0, 0);
            } else {
                lcd_set_color(0, 63, 255);
            }
            lcd_print_raw(cpuName, 4 * scr_scale, 3 * scr_scale, scr_scale);
            lcd_set_color(255, 255, 255);

            lcd_print_raw("Freq GHz",      0 * scr_scale, 28 * scr_scale, scr_scale);
            lcd_print_raw("Load %",       68 * scr_scale, 28 * scr_scale, scr_scale);
            lcd_print_raw("Temp \xF7 C", 112 * scr_scale, 28 * scr_scale, scr_scale);

            // dividers
            // TODO: replace with lcd_rect calls
            // tft.drawLine(51 * scr_scale, 12 * scr_scale, 51 * scr_scale, 34 * scr_scale, ST77XX_WHITE);
            // tft.drawLine(106 * scr_scale, 12 * scr_scale, 106 * scr_scale, 34 * scr_scale, ST77XX_WHITE);

            if (strncmp(cpuName, "AMD", 3) == 0) {
                lcd_set_color(255, 0, 0);
            } else {
                lcd_set_color(63, 255, 63);
            }
            lcd_print_raw(gpuName, 4 * scr_scale, 47 * scr_scale, scr_scale);
            lcd_set_color(255, 255, 255);

            lcd_print_raw("Load %", 12 * scr_scale, 72 * scr_scale, scr_scale);
            lcd_print_raw("VRAM %", 68 * scr_scale, 72 * scr_scale, scr_scale);

            lcd_print_raw("Core MHz",  0 * scr_scale, 96 * scr_scale, scr_scale);
            lcd_print_raw("VRAM MHz", 56 * scr_scale, 96 * scr_scale, scr_scale);
            lcd_print_raw("Temp \xF7 C",112 * scr_scale, 72 * scr_scale, scr_scale);
            
            // dividers
            // TODO: replace with lcd_rect calls
            // tft.drawLine(51 * scr_scale, 56 * scr_scale, 51 * scr_scale, 104 * scr_scale, ST77XX_WHITE);
            // tft.drawLine(106 * scr_scale, 56 * scr_scale, 106 * scr_scale, 104 * scr_scale, ST77XX_WHITE);

            // lcd_print_raw("RAM: " + ramCount, scr_width - 6 * 10 * scr_scale, scr_height - 8 * scr_scale, scr_scale);
        }
    }

    // Repeat draws!
    if (screenstate == ShowStats) {
        lcd_print_raw(cpuFreq,   0 * scr_scale, 12 * scr_scale, 2 * scr_scale);
        lcd_print_raw(cpuLoad,  56 * scr_scale, 12 * scr_scale, 2 * scr_scale);
        lcd_print_raw(cpuTemp, 112 * scr_scale, 12 * scr_scale, 2 * scr_scale);

        lcd_print_raw(gpuCoreLoad,   0 * scr_scale, 56 * scr_scale, 2 * scr_scale);
        lcd_print_raw(gpuVramLoad,  56 * scr_scale, 56 * scr_scale, 2 * scr_scale);
        lcd_print_raw(gpuTemp,     112 * scr_scale, 56 * scr_scale, 2 * scr_scale);

        lcd_print_raw(gpuCoreClock, 12 * scr_scale, 88 * scr_scale, scr_scale);
        lcd_print_raw(gpuVramClock, 68 * scr_scale, 88 * scr_scale, scr_scale);

        // lcd_print_raw(ramUsed, scr_width - 5 * 10 * scr_scale, scr_height - 24 * scr_scale, 2 * scr_scale);

        // countermax = 1000;
    }
}

void lcd_draw_task(void) {
    REFRESH_CHECK(JB_SCREEN_DRAW_INTERVAL, JB_SCREEN_DRAW_OFFSET);
    
    lcd_present();
    lcd_clear();
}
