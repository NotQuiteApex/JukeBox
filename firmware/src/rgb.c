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

uint32_t pixel_buf[JB_RGBLEDS_NUM] = {0};

void rgb_init(void) {
	rgb_offset = pio_add_program(rgb_pio, &ws2812_program);
	ws2812_program_init(rgb_pio, rgb_sm, rgb_offset, JB_RGBLEDS_PIN, JB_RGBLEDS_FREQ, is_rgbw);
}

uint32_t rgb_to_grbw(uint8_t r, uint8_t g, uint8_t b, uint8_t w) {
	return ((uint32_t) (r) << 16) | ((uint32_t) (g) << 24) | ((uint32_t) (b) << 8)| ((uint32_t) (w));
}

uint32_t hsv_to_grbw(uint16_t hue, uint8_t sat, uint8_t val) {
	uint8_t r, g, b;
	hue = (hue * 1530L + 32768) / 65536;
	if (hue < 510) {
    	b = 0;
		if (hue < 255) {
			r = 255;
			g = hue;
		} else {
			r = 510 - hue;
			g = 255;
		}
	} else if (hue < 1020) {
		r = 0;
		if (hue < 765) {
			g = 255;
			b = hue - 510;
		} else {
			g = 1020 - hue;
			b = 255;
		}
	} else if (hue < 1530) {
		g = 0;
		if (hue < 1275) {
			r = hue - 1020;
			b = 255;
		} else {
			r = 255;
			b = 1530 - hue;
		}
	} else {
		r = 255;
		g = b = 0;
	}

	uint32_t v1 = 1 + val;
	uint16_t s1 = 1 + sat;
	uint8_t s2 = 255 - sat;

	return rgb_to_grbw(
		((((r * s1) >> 8) + s2) * v1) >> 8,
		((((g * s1) >> 8) + s2) * v1) >> 8,
		((((b * s1) >> 8) + s2) * v1) >> 8,
		0
	);
}

void rgb_put(uint8_t idx, uint32_t pixel_grbw) {
	if (idx >= JB_RGBLEDS_NUM) {
		return;
	}
	pixel_buf[idx] = pixel_grbw;
}

void rgb_clear(void) {
	for (uint8_t i = 0; i < JB_RGBLEDS_NUM; i++) {
		rgb_put(i, 0);
	}
}

void rgb_present(void) {
	for (uint8_t i = 0; i < JB_RGBLEDS_NUM; i++) {
		pio_sm_put_blocking(rgb_pio, rgb_sm, pixel_buf[i]);
	}
}

void rgb_task(void) {
	REFRESH_CHECK(JB_RGBLEDS_REFRESH_INTERVAL, JB_RGBLEDS_REFRESH_OFFSET);

	if (tud_suspended()) {
		rgb_clear();
		rgb_present();
		return;
	}

	rgb_clear();

	for (uint8_t i = 0; i < JB_RGBLEDS_NUM; i++) {
		rgb_put(i, hsv_to_grbw((time_us_32()>>8) - 512*i, 255, 75));
	}

	rgb_present();
}
