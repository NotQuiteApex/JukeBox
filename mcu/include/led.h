#include <pico/stdlib.h>

#ifndef JUKEBOX_LED_H
#define JUKEBOX_LED_H

void led_init(void);
void led_blinking_task(void);

void led_set_mounted(void);
void led_set_unmounted(void);
void led_set_suspended(void);


#endif // JUKEBOX_LED_H
