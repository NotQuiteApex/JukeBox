#include "common.h"
#include <hardware/pio.h>

#ifndef JUKEBOX_WS2812_RGB_H
#define JUKEBOX_WS2812_RGB_H

void rgb_init(void);
void rgb_put_pixel(uint32_t pixel_grb);
void rgb_task(void);

#endif // JUKEBOX_WS2812_RGB_H
