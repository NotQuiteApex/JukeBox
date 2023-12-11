#include "common.h"

#include <tusb.h>

#ifndef JUKEBOX_KEYBOARD_H
#define JUKEBOX_KEYBOARD_H

void keyboard_init(void);
void keyboard_send_hid_report(uint8_t);
void keyboard_task(void);

#endif // JUKEBOX_KEYBOARD_H
