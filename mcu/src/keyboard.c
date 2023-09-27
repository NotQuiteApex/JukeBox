#include "keyboard.h"

#define _NOP() do { __asm__ __volatile__ ("nop"); } while (0)
#define _NOPS(n) do { for(uint8_t _NOP_LOOP_I=0; _NOP_LOOP_I<n; _NOP_LOOP_I++) { _NOP(); } } while (0)
#define KB_NOP_COUNT 24

const uint COL     = JB_HID_KB_COL;
const uint COL_NUM = JB_HID_KB_COL_NUM;
const uint ROW     = JB_HID_KB_ROW;
const uint ROW_NUM = JB_HID_KB_ROW_NUM;


void keyboard_init(void) {
    // setup keyboard pins
    // column pins are pulldown inputs. if their current state is high, then the key is down.
    for (uint8_t col=0; col<COL_NUM; col++) {
        gpio_init(COL + col);
        gpio_set_dir(COL + col, GPIO_IN);
        gpio_pull_down(COL + col);
    }
    // row pins are outputs. they produce current to check each row of keys.
    for (uint8_t row=0; row<ROW_NUM; row++) {
        gpio_init(ROW + row);
        gpio_set_dir(ROW + row, GPIO_OUT);
        gpio_put(ROW + row, 0);
    }
}


void keyboard_send_hid_report(uint8_t report_id) {
    // skip if hid is not ready yet
    if (!tud_hid_ready() && report_id != REPORT_ID_KEYBOARD) {
        return;
    }

    static uint8_t keys [] = {
        HID_KEY_F13, HID_KEY_F14, HID_KEY_F15, HID_KEY_F16,
        HID_KEY_F17, HID_KEY_F18, HID_KEY_F19, HID_KEY_F20,
        HID_KEY_F21, HID_KEY_F22, HID_KEY_F23, HID_KEY_F24,
    };

    uint8_t k = 0;
    uint8_t keycodes[6] = {0, 0, 0, 0, 0, 0};
    uint8_t usedKeys = 0;
    static bool pressedKey = false;

    for (uint8_t row=0; row<ROW_NUM; row++) {
        gpio_put(ROW + row, 1);
        _NOPS(KB_NOP_COUNT);
        for (uint8_t col=0; col<COL_NUM; col++) {
            if (usedKeys >= 6) {
                continue;
            }

            if (gpio_get(COL + col)) {
                keycodes[usedKeys++] = keys[k];
            }

            k++;
        }
        gpio_put(ROW + row, 0);
    }

    if (tud_suspended() && usedKeys > 0) {
        tud_remote_wakeup();
    } else if (usedKeys > 0) {
        tud_hid_keyboard_report(REPORT_ID_KEYBOARD, 0, keycodes);
        pressedKey = true;
    } else {
        if (pressedKey) {
            tud_hid_keyboard_report(REPORT_ID_KEYBOARD, 0, NULL);
        }
        pressedKey = false;
    }
}

void keyboard_task(void) {
    REFRESH_CHECK(JB_HID_REFRESH_INTERVAL, JB_HID_REFRESH_OFFSET);

    keyboard_send_hid_report(REPORT_ID_KEYBOARD);
}
