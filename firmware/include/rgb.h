#include "common.h"
#include <hardware/pio.h>

#ifndef JUKEBOX_WS2812_RGB_H
#define JUKEBOX_WS2812_RGB_H

void rgb_init(void);

uint32_t rgb_to_grbw(uint8_t r, uint8_t g, uint8_t b, uint8_t w);
uint32_t hsv_to_grbw(uint16_t hue, uint8_t sat, uint8_t val);

void rgb_put(uint8_t idx, uint32_t pixel_grb);
void rgb_clear(void);
void rgb_present(void);
void rgb_task(void);

#endif // JUKEBOX_WS2812_RGB_H
