
#ifndef JUKEBOX_COMMON_H
#define JUKEBOX_COMMON_H

// Common includes
#include <pico/stdlib.h>
#include "config.h"


// Common macros
#define BIT(n) (1<<n)

#define REFRESH_CHECK(X, Y) \
	do { \
		static uint64_t start_ms = 0; \
		if ( time_us_64() / 1000 - start_ms - Y < X) { \
			return; \
		} \
		start_ms += X; \
	} while (0);

#ifndef MAX
	#define MAX(X, Y) (((X) > (Y)) ? (X) : (Y))
#endif
#ifndef MIN
	#define MIN(X, Y) (((X) < (Y)) ? (X) : (Y))
#endif


// Config checking
// TODO

#if JB_HID_REFRESH_INTERVAL > 250
	#error "HID refresh interval must be greater than 250 ms."
#endif
#if JB_SERIAL_REFRESH_INTERVAL > 250
	#error "Serial refresh interval must be greater than 250 ms."
#endif

#ifdef JB_MOD_SCREEN
	#if JB_SCREEN_REFRESH_INTERVAL > 250
		#error "Screen framebuffer refresh interval must be greater than 250 ms."
	#endif

	#if JB_SCREEN_PIN_DIN > 22
		#error "Screen pin DIN must be 22 or less."
	#elif JB_SCREEN_PIN_CLK > 22
		#error "Screen pin CLK must be 22 or less."
	#elif JB_SCREEN_PIN_CS > 22
		#error "Screen pin CS must be 22 or less."
	#elif JB_SCREEN_PIN_DC > 22
		#error "Screen pin DC must be 22 or less."
	#elif JB_SCREEN_PIN_RST > 22
		#error "Screen pin RST must be 22 or less."
	#elif JB_SCREEN_PIN_BL > 22
		#error "Screen pin BL must be 22 or less."
	#endif

	#define JB_PORTRAIT 1
	#define JB_LANDSCAPE 2
	#if JB_SCREEN_ORIENTATION != JB_PORTRAIT && JB_SCREEN_ORIENTATION != JB_LANDSCAPE
		#error "Screen orientation must be either JB_PORTRAIT or JB_LANDSCAPE"
	#endif
#endif

#ifdef JB_MOD_RGBLEDS
	#if JB_RGBLEDS_REFRESH_INTERVAL > 1000
		#error "Screen framebuffer refresh interval must be greater than 1000 ms."
	#endif
	
	#if JB_RGBLEDS_PIN > 22
		#error "RGB LED pin must be 22 or less."
	#endif
#endif

#endif // JUKEBOX_COMMON_H
