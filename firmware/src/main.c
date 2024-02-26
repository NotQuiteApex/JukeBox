// JukeBox V5 Firmware

#include "common.h"

#include <pico/multicore.h>
#include <pico/bootrom.h>

#include "keyboard.h"
#include "lcd.h"
#include "led.h"
#include "rgb.h"
#include "serial.h"


volatile uint8_t bootsel_reset_jukebox = 0;
volatile uint8_t core1_finish_jukebox = 0;


void task_updates(void) {
	while (!bootsel_reset_jukebox) {
		serial_task();
		if (bootsel_reset_jukebox) {
			break;
		}

		#ifdef JB_MOD_SCREEN
			lcd_task();
		#endif

		#ifdef JB_MOD_RGBLEDS
			rgb_task();
		#endif
	}

	core1_finish_jukebox = 1;
}


int main(void) {
	led_init();
	keyboard_init();

	serial_init();

	#ifdef JB_MOD_SCREEN
		lcd_init();
	#endif
	#ifdef JB_MOD_RGBLEDS
		rgb_init();
	#endif

	tusb_init();
	bool started_tasks = false;

	while (true) {
		tud_task();

		if (tud_mounted()) {
			if (!started_tasks) {
				started_tasks = true;
				multicore_launch_core1(task_updates);
			}
			keyboard_task();
		}

		if (bootsel_reset_jukebox) {
			// TODO: set activity led? wait for second core to finish?
			while (!core1_finish_jukebox) { /* wait until the second core exits */ }
			lcd_off();
			lcd_clear();
			lcd_present();
			rgb_clear();
			rgb_present();
			reset_usb_boot(0, 0);
		}

		led_blinking_task();
	}

	return 0;
}
