#include "common.h"
#include <hardware/pio.h>

#include "lcd.h"

#ifndef JUKEBOX_SERIAL_H
#define JUKEBOX_SERIAL_H

void serial_init(void);
void serial_task(void);

typedef enum
{
	ErrorWait,
	GreetHost,
	GreetDevice,
	LinkConfirmHost,
	LinkConfirmDevice,
	TransmitReady,
} SerialStage;

extern char inputString[65];
extern uint8_t inputStringLen;
extern uint8_t inputStringReady;

extern char cpuName[28];
extern char gpuName[28];
extern char ramCount[6];

extern char cpuFreq[6];
extern char cpuTemp[6];
extern char cpuLoad[6];
extern char ramUsed[6];
extern char gpuTemp[6];
extern char gpuCoreClock[8];
extern char gpuCoreLoad[6];
extern char gpuVramClock[8];
extern char gpuVramLoad[6];

#endif // JUKEBOX_SERIAL_H
