#include "serial.h"

#include <string.h>
#include <tusb.h>

extern ScreenState screenstate;
SerialStage commstage = Handshake;

// The parts of comm stages (stage recieve and response)
int commstagepart = 0;

// The data string we use to store received data
// TODO: BOOST TO 128
char inputString[64] = "";
char sentString[10] = "";

// Data we store!
// Data that we receive only at the time a connection is made.
char cpuName[28] = "";
char gpuName[28] = "";
char ramCount[6] = "";
// Data we constantly receive after a connection is made.
char cpuFreq[6] = "";
char cpuTemp[6] = "";
char cpuLoad[6] = "";
char ramUsed[6] = "";
char gpuTemp[6] = "";
char gpuCoreClock[8] = "";
char gpuCoreLoad[6]  = "";
char gpuVramClock[8] = "";
char gpuVramLoad[6]  = "";

uint32_t countermax;

void serial_init(void) {
  
}

void serial_task(void) {
  // check serial string
  if (tud_cdc_available()) {
    // read datas
    uint32_t count = tud_cdc_read(inputString, sizeof(inputString)-1);
    inputString[count] = '\0';
  }
  
  REFRESH_CHECK(countermax, JB_SERIAL_REFRESH_OFFSET);
  
  // comms management
  static uint8_t errorcount = 0;
  // we mustve lost comms, reset!
  if (errorcount >= 5) {
    errorcount = 0;

    commstage = Handshake;
    commstagepart = 0;

    screenstate = WaitingConnection;

    inputString[0] = '\0';
    sentString[0] = '\0';

    cpuName[0] = '\0';
    gpuName[0] = '\0';
    ramCount[0] = '\0';

    cpuFreq[0] = '\0';
    cpuTemp[0] = '\0';
    cpuLoad[0] = '\0';
    ramUsed[0] = '\0';
    gpuTemp[0] = '\0';
    gpuCoreClock[0] = '\0';
    gpuCoreLoad[0] = '\0';
    gpuVramClock[0] = '\0';
    gpuVramLoad[0] = '\0';
  }

  if (commstage == Handshake) {
    if (commstagepart == 0) {
      if (strncmp(inputString, "010", 3) == 0) {
        inputString[0] = '\0';
        commstagepart = 1;
      }
    } else if (commstagepart == 1) {
      // TODO: double check that this works as expected
      strncpy(sentString, "101", 4);
      tud_cdc_write("101", 4);
      tud_cdc_write_flush();
      commstage = ComputerParts;
      commstagepart = 0;
    }
  } else if (commstage == ComputerParts) {
    if (commstagepart == 0) {
      // wait for info on CPU, GPU, and RAM
      if (receive_once_data()) {
        commstagepart = 1;
        errorcount = 0;
      }
      else {
        errorcount++;
        countermax = 1000;
      }
    } else if (commstagepart == 1) {
      // respond that we got it, and move to the next stage
      // TODO: double check that this works as expected
      strncpy(sentString, "222", 4);
      tud_cdc_write("222", 4);
      tud_cdc_write_flush();
      commstage = ContinuousStats;
      screenstate = ShowStats;
      commstagepart = 0;
    }
  } else if (commstage == ContinuousStats) {
    if (commstagepart == 0) {
      // recieve
      if (receive_cont_data()) {
        commstagepart = 1;
        errorcount = 0;
      } else {
        errorcount++;
        countermax = 1000;
      }
    } else if (commstagepart == 1) {
      // respond that we got it, and repeat
      // TODO: double check that this works as expected
      strncpy(sentString, "333", 4);
      tud_cdc_write("333", 4);
      tud_cdc_write_flush();
      commstagepart = 0;
    }
  }
}

uint8_t receive_once_data() {
  // string format of computer parts stats
  // $"{cpuName}|{gpuName}|{ramTotal}GB|"

  // if data isn't empty, process it
  if (inputString[0] != '\0') {
    // process the names!
    // beginning index and end index, and the variable to store data into
    uint8_t idx1 = 0;
    uint8_t idx2 = 0;
    uint8_t count = 0;

    // Loop through the string, finding the pipe seperators
    for (uint8_t i = 0; i < strnlen(inputString, sizeof(inputString)); i++) {
      // if character matches a pipe, we process the data before
      if (inputString[i] == '|') {
        // Old start index, to end index
        idx1 = idx2;
        if (inputString[idx1] == '|') {
          idx1++;
        }
        idx2 = i;

        // hand off data to proper variable
        if (count == 0) {
          strncpy(cpuName, inputString+idx1, MIN(sizeof(cpuName), idx2-idx1));
        } else if (count == 1) {
          strncpy(gpuName, inputString+idx1, MIN(sizeof(gpuName), idx2-idx1));
        } else if (count == 2) {
          strncpy(ramCount, inputString+idx1, MIN(sizeof(ramCount), idx2-idx1));
        }

        // update variable to determine what variable to hand off to next
        count++;
      }
    }
    
    inputString[0] = '\0';
    return 1; // we sucessfully processed data
  }

  return 0; // we didn't have any data to process, we bounce
}

uint8_t receive_cont_data() {
  // Format string of continuous stats
  // $"{cpuFreq}|{cpuTemp}|{cpuLoad}|{ramUsed}|{gpuTemp}|" +
  // $"{gpuCoreClock}|{gpuCoreLoad}|{gpuVramClock}|{gpuVramLoad}|"

  // this is the same as above, just with more variables to hand data to

  if (inputString[0] != '\0') {
    uint8_t idx1 = 0;
    uint8_t idx2 = 0;
    uint8_t count = 0;

    for (uint8_t i = 0; i < strnlen(inputString, sizeof(inputString)); i++) {
      if (inputString[i] == '|') {
        idx1 = idx2;
        if (inputString[idx1] == '|') {
          idx1++;
        }
        idx2 = i;

        if (count == 0) {
          strncpy(cpuFreq, inputString+idx1, MIN(sizeof(cpuFreq), idx2-idx1));
        } else if (count == 1) {
          strncpy(cpuTemp, inputString+idx1, MIN(sizeof(cpuTemp), idx2-idx1));
        } else if (count == 2) {
          strncpy(cpuLoad, inputString+idx1, MIN(sizeof(cpuLoad), idx2-idx1));
        } else if (count == 3) {
          strncpy(ramUsed, inputString+idx1, MIN(sizeof(ramUsed), idx2-idx1));
        } else if (count == 4) {
          strncpy(gpuTemp, inputString+idx1, MIN(sizeof(gpuTemp), idx2-idx1));
        } else if (count == 5) {
          strncpy(gpuCoreClock, inputString+idx1, MIN(sizeof(gpuCoreClock), idx2-idx1));
        } else if (count == 6) {
          strncpy(gpuCoreLoad, inputString+idx1, MIN(sizeof(gpuCoreLoad), idx2-idx1));
        } else if (count == 7) {
          strncpy(gpuVramClock, inputString+idx1, MIN(sizeof(gpuVramClock), idx2-idx1));
        } else if (count == 8) {
          strncpy(gpuVramLoad, inputString+idx1, MIN(sizeof(gpuVramLoad), idx2-idx1));
        }

        count++;
      }
    }

    inputString[0] = '\0';
    return 1;
  }

  return 0;
}
