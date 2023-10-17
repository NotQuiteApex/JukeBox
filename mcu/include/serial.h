#include "common.h"
#include <hardware/pio.h>

#include "lcd.h"

#ifndef JUKEBOX_SERIAL_H
#define JUKEBOX_SERIAL_H

void serial_begin(void);
void serial_loop(void);
void serial_receive(void);
bool serial_matches(char *);
bool receive_once_data(void);
bool receive_cont_data(void);

#endif // JUKEBOX_SERIAL_H
