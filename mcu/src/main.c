#include <stdio.h>
#include "pico/stdlib.h"

#ifndef PICO_DEFAULT_LED_PIN
#warning blink example requires a board with a regular LED
#else
    const uint LED_PIN = PICO_DEFAULT_LED_PIN;
#endif

bool blink_timer(struct repeating_timer * t) {
#ifdef PICO_DEFAULT_LED_PIN
    static bool LED_FLASH = false;
    LED_FLASH = !LED_FLASH;
    gpio_put(LED_PIN, LED_FLASH ? 1 : 0);
#endif
    return true;
}

int main() {
    setup_default_uart();
    printf("Hello, world!\n");

#ifdef PICO_DEFAULT_LED_PIN
    gpio_init(LED_PIN);
    gpio_set_dir(LED_PIN, GPIO_OUT);
#endif

    struct repeating_timer timer;
    add_repeating_timer_ms(-1000, blink_timer, NULL, &timer);

    while (true) {}

    return 0;
}
