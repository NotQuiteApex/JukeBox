// JukeBox V5 Firmware

#include "common.h"
#include <pico/multicore.h>

#include "keyboard.h"
#include "lcd.h"
#include "led.h"
#include "rgb.h"
#include "serial.h"

void task_updates(void) {
	while (true) {
		keyboard_task();
		serial_task();

		#ifdef JB_MOD_SCREEN
			lcd_task();
		#endif

		#ifdef JB_MOD_RGBLEDS
			rgb_task();
		#endif
	}
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

		if (!started_tasks && tud_mounted()) {
			started_tasks = true;
			multicore_launch_core1(task_updates);
		}

		led_blinking_task();
	}

	return 0;
}
