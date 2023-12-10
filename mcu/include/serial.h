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

extern char inputString[128];
extern uint16_t inputStringLen;
extern uint8_t inputStringReady;

extern char cpuName[48];
extern char gpuName[48];
extern char ramCount[10];

extern char cpuFreq[8];
extern char cpuTemp[8];
extern char cpuLoad[8];
extern char ramUsed[8];
extern char gpuTemp[8];
extern char gpuCoreClock[10];
extern char gpuCoreLoad[8];
extern char gpuVramClock[10];
extern char gpuVramLoad[8];

#endif // JUKEBOX_SERIAL_H
