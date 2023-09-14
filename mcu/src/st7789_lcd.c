#include "st7789_lcd.h"

#include <hardware/gpio.h>

#include "st7789_lcd.pio.h"


// Tested with the parts that have the height of 240 and 320
#define SCREEN_WIDTH 240
#define SCREEN_HEIGHT 320

#define PIN_DIN 0
#define PIN_CLK 1
#define PIN_CS 2
#define PIN_DC 3
#define PIN_RESET 4
#define PIN_BL 5

#define SERIAL_CLK_DIV 1.f

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif


PIO pio = pio0;
uint sm = 0;
uint offset = 0;

// Format: cmd length (including cmd byte), post delay in units of 5 ms, then cmd payload
// Note the delays have been shortened a little
static const uint8_t st7789_init_seq[] = {
    1, 20, 0x01,                        // Software reset
    1, 10, 0x11,                        // Exit sleep mode
    2, 2, 0x3A, 0x55,                   // Set colour mode to 16 bit
    2, 0, 0x36, 0x00,                   // Set MADCTL: row then column, refresh is bottom to top ????
    5, 0, 0x2A, 0x00, 0x00, SCREEN_WIDTH >> 8, SCREEN_WIDTH & 0xFF,   // CASET: column addresses
    5, 0, 0x2B, 0x00, 0x00, SCREEN_HEIGHT >> 8, SCREEN_HEIGHT & 0xFF, // RASET: row addresses
    1, 2, 0x21,                         // Inversion on, then 10 ms delay (supposedly a hack?)
    1, 2, 0x13,                         // Normal display on, then 10 ms delay
    1, 2, 0x29,                         // Main screen turn on, then wait 500 ms
    0                                   // Terminate list
};

static uint16_t framebuffer[SCREEN_HEIGHT][SCREEN_WIDTH];


void lcd_set_dc_cs(bool dc, bool cs) {
    sleep_us(1);
    gpio_put_masked((1u << PIN_DC) | (1u << PIN_CS), !!dc << PIN_DC | !!cs << PIN_CS);
    sleep_us(1);
}


void lcd_write_cmd(PIO pio, uint sm, const uint8_t *cmd, size_t count) {
    st7789_lcd_wait_idle(pio, sm);
    lcd_set_dc_cs(0, 0);
    st7789_lcd_put(pio, sm, *cmd++);
    if (count >= 2) {
        st7789_lcd_wait_idle(pio, sm);
        lcd_set_dc_cs(1, 0);
        for (size_t i = 0; i < count - 1; ++i)
            st7789_lcd_put(pio, sm, *cmd++);
    }
    st7789_lcd_wait_idle(pio, sm);
    lcd_set_dc_cs(1, 1);
}


void st7789_lcd_init(void) {
    offset = pio_add_program(pio, &st7789_lcd_program);
    st7789_lcd_program_init(pio, sm, offset, PIN_DIN, PIN_CLK, SERIAL_CLK_DIV);

    gpio_init(PIN_CS);
    gpio_init(PIN_DC);
    gpio_init(PIN_RESET);
    gpio_init(PIN_BL);
    gpio_set_dir(PIN_CS, GPIO_OUT);
    gpio_set_dir(PIN_DC, GPIO_OUT);
    gpio_set_dir(PIN_RESET, GPIO_OUT);
    gpio_set_dir(PIN_BL, GPIO_OUT);

    gpio_put(PIN_CS, 1);
    gpio_put(PIN_RESET, 1);

    const uint8_t *cmd = st7789_init_seq;
    while (*cmd) {
        lcd_write_cmd(pio, sm, cmd + 2, *cmd);
        sleep_ms(*(cmd + 1) * 5);
        cmd += *cmd + 2;
    }

    gpio_put(PIN_BL, 1);

    st7789_fb_clear();
}


void st7789_start_pixels(PIO pio, uint sm) {
    uint8_t cmd = 0x2c; // RAMWR
    lcd_write_cmd(pio, sm, &cmd, 1);
    lcd_set_dc_cs(1, 0);
}

void st7789_fb_clear(void) {
    for (uint16_t y=0; y<SCREEN_HEIGHT; y++) {
        for (uint16_t x=0; x<SCREEN_WIDTH; x++) {
            framebuffer[y][x] = 0;
        }
    }
}

void st7789_fb_put(uint16_t color, uint16_t x, uint16_t y) {
    if (x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT) {
        // if its off screen, whatever
        return;
    }

    framebuffer[y][x] = color;
}

void st7789_lcd_push_fb(void) {
    st7789_start_pixels(pio, sm);
    uint16_t color = 0;
    for (uint16_t y=0; y<SCREEN_HEIGHT; y++) {
        for (uint16_t x=0; x<SCREEN_WIDTH; x++) {
            color = framebuffer[y][x];
            st7789_lcd_put(pio, sm, color >> 8);
            st7789_lcd_put(pio, sm, color & 0xFF);
        }
    }
}

inline uint16_t st7789_get_width(void) {
    return SCREEN_WIDTH;
}

inline uint16_t st7789_get_height(void) {
    return SCREEN_HEIGHT;
}
