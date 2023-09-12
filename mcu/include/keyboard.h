#include <pico/stdlib.h>
#include <tusb.h>

#include "usb_descriptors.h"

#ifndef JUKEBOX_KEYBOARD_H
#define JUKEBOX_KEYBOARD_H

void keyboard_init(void);
void keyboard_send_hid_report(uint8_t);

#endif // JUKEBOX_KEYBOARD_H
