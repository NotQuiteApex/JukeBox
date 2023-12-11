#include "common.h"
#include <hardware/pio.h>

#ifndef JUKEBOX_ST7789_LCD_H
#define JUKEBOX_ST7789_LCD_H

void lcd_set_dc_cs(bool dc, bool cs);
void lcd_write_cmd(PIO pio, uint sm, const uint8_t *cmd, size_t count);
void st7789_lcd_init(void);
void st7789_start_pixels(PIO pio, uint sm);
void st7789_fb_clear(void);
void st7789_fb_put(uint16_t color, uint16_t x, uint16_t y);
void st7789_lcd_push_fb(void);

uint16_t st7789_get_width(void);
uint16_t st7789_get_height(void);

#endif // JUKEBOX_ST7789_LCD_H
