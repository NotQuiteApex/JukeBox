#include "rgb.h"

#include <hardware/gpio.h>

#include "ws2812_rgb.pio.h"

#ifdef JB_RGBLEDS_IS_RGBW
    const bool is_rgbw = true;
#else
    const bool is_rgbw = false;
#endif

PIO rgb_pio = pio1;
int rgb_sm = 1;
uint rgb_offset = 0;

void rgb_init(void) {
    rgb_offset = pio_add_program(rgb_pio, &ws2812_program);
    ws2812_program_init(rgb_pio, rgb_sm, rgb_offset, JB_RGBLEDS_PIN, JB_RGBLEDS_FREQ, is_rgbw);
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
