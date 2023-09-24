#include "lcd.h"

#include <string.h>

#include "font.h"
#include "st7789_lcd.h"


inline uint16_t lcd_rgb565(uint8_t r, uint8_t g, uint8_t b) {
    // https://stackoverflow.com/a/76442697/13977827
    return ((r>>3) << 11) | ((g>>2) << 5) | b >> 3;
}

inline void lcd_init(void) {
    st7789_lcd_init();
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

void lcd_task(void) {
    REFRESH_CHECK(JB_SCREEN_REFRESH_INTERVAL, JB_SCREEN_REFRESH_OFFSET);

    lcd_set_color(255, 0, 0);
    lcd_print_raw("Testing 0123 !", 5, 5, 1);
    lcd_set_color(0, 255, 255);
    lcd_print_raw("\xd\xe \x2 \x3 \x1 \xd\xe", 5, 30, 1);

    lcd_set_color(255, 255, 255);
    lcd_print_raw("Say hello to the new", 25, 80, 1);
    lcd_set_color(255, 170, 70);
    lcd_print_raw("JukeBox V5 by F.T.I.!", 25, 100, 1);
}

void lcd_draw_task(void) {
    REFRESH_CHECK(JB_SCREEN_DRAW_INTERVAL, JB_SCREEN_DRAW_OFFSET);
    
    lcd_present();
    lcd_clear();
}
