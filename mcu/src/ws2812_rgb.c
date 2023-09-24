#include "ws2812_rgb.h"

#include <hardware/gpio.h>

#include "ws2812_rgb.pio.h"

#define WS2812_PIN 6
#define WS2812_FREQ 800000.f
#define WS2812_IS_RGBW false

PIO rgb_pio = pio1;
int rgb_sm = 1;
uint rgb_offset = 0;

void rgb_init(void) {
    rgb_offset = pio_add_program(rgb_pio, &ws2812_program);
    ws2812_program_init(rgb_pio, rgb_sm, rgb_offset, WS2812_PIN, WS2812_FREQ, WS2812_IS_RGBW);
}

void rgb_put_pixel(uint32_t pixel_grb) {
    pio_sm_put_blocking(rgb_pio, rgb_sm, pixel_grb << 8u);
}

void rgb_task(void) {
    REFRESH_CHECK(JB_RGBLEDS_REFRESH_INTERVAL, JB_RGBLEDS_REFRESH_OFFSET);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040000);
    rgb_put_pixel(0x000400);
    rgb_put_pixel(0x000004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040400);
    rgb_put_pixel(0x000404);
    rgb_put_pixel(0x040004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x040000);
    rgb_put_pixel(0x000400);
    rgb_put_pixel(0x000004);

    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
    rgb_put_pixel(0x000000);
}
