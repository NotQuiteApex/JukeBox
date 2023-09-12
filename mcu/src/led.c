#include "led.h"

// Blink pattern
enum  {
  BLINK_NOT_MOUNTED = 100,
  BLINK_MOUNTED = 500,
  BLINK_SUSPENDED = 1000,
};
static uint32_t blink_interval_ms = BLINK_NOT_MOUNTED;


void led_init(void) {
    gpio_init(PICO_DEFAULT_LED_PIN);
    gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
}


void led_blinking_task(void) {
    static uint32_t start_ms = 0;
    static bool led_state = false;

    // blink is disabled
    if (!blink_interval_ms) {
        return;
    }

    // Blink every interval ms
    if ( time_us_64() / 1000 - start_ms < blink_interval_ms) {
        return; // not enough time
    }
    start_ms += blink_interval_ms;

    gpio_put(PICO_DEFAULT_LED_PIN, led_state);
    led_state = 1 - led_state; // toggle
}


void led_set_mounted(void) {
    blink_interval_ms = BLINK_MOUNTED;
}

void led_set_unmounted(void) {
    blink_interval_ms = BLINK_NOT_MOUNTED;
}

void led_set_suspended(void) {
    blink_interval_ms = BLINK_SUSPENDED;
}
