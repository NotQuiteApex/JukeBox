#include "common.h"
#include <hardware/pio.h>

#include "lcd.h"

#ifndef JUKEBOX_SERIAL_H
#define JUKEBOX_SERIAL_H

void serial_init(void);
void serial_task(void);

uint8_t receive_once_data(void);
uint8_t receive_cont_data(void);

// Stages of serial communication
typedef enum
{
  Handshake,      // The initial stage, where the computer and MCU greet
  ComputerParts,  // The computer sends some constant data over the wire
  ContinuousStats // The computer sends data that gets updated over time
} SerialStage;

extern char inputString[64];
extern char sentString[10];

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
