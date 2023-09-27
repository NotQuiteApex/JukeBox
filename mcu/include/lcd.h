#include "common.h"

#ifndef JUKEBOX_LCD_H
#define JUKEBOX_LCD_H

void lcd_init(void);

void lcd_set_color(uint8_t r, uint8_t g, uint8_t b);

void lcd_clear(void);
void lcd_present(void);

void lcd_put(uint16_t x, uint16_t y);

void lcd_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h);

void lcd_print(char * text, uint16_t x, uint16_t y, uint8_t s);
void lcd_print_raw(char * text, uint16_t x, uint16_t y, uint8_t s);

void cdc_task(void);
void lcd_task(void);
void lcd_draw_task(void);

#endif // JUKEBOX_LCD_H
