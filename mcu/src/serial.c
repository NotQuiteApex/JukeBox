#include "serial.h"

#include <string.h>
#include <tusb.h>

extern ScreenState screenstate;
SerialStage commstage = GreetHost;

// The parts of comm stages (stage recieve and response)
int commstagepart = 0;

// The data string we use to store received data
// TODO: BOOST TO 128
char inputString[65] = "";
uint8_t inputStringLen = 0;
uint8_t inputStringReady = 0;

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

uint32_t countermax = 0;

inline void reset_input_string(void) {
  inputString[0] = '\0';
  inputStringLen = 0;
  inputStringReady = 0;
}

inline void reset_state_data(void) {
  commstage = GreetHost;

  screenstate = WaitingConnection;

  reset_input_string();

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

uint8_t parse_pc_part_info(void) {
  // process the names!
  // string format of computer parts stats
  // $"{cpuName}|{gpuName}|{ramTotal}GB|"

  // beginning index and end index, and the variable to store data into
  uint8_t idx1 = 3;
  uint8_t idx2 = 3;
  uint8_t count = 0;

  // Loop through the string, finding the unit seperators
  for (uint8_t i = 0; i < strnlen(inputString, sizeof(inputString)); i++) {
    // if character matches a unit separator, we process the data before
    if (inputString[i] == '\x1F') {
      // Old start index, to end index
      idx1 = idx2;
      if (inputString[idx1] == '\x1F') {
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

  if (count != 3) {
    // TODO: reset input string on failed parse? display an error? figure this out.
    return 0;
  }
  
  return 1; // we sucessfully processed data
}

uint8_t parse_pc_part_stats(void) {
  // Format string of continuous stats
  // $"{cpuFreq}|{cpuTemp}|{cpuLoad}|{ramUsed}|{gpuTemp}|" +
  // $"{gpuCoreClock}|{gpuCoreLoad}|{gpuVramClock}|{gpuVramLoad}|"

  // this is the same as above, just with more variables to hand data to

  uint8_t idx1 = 3;
  uint8_t idx2 = 3;
  uint8_t count = 0;

  for (uint8_t i = 0; i < strnlen(inputString, sizeof(inputString)); i++) {
    if (inputString[i] == '\x1F') {
      idx1 = idx2;
      if (inputString[idx1] == '\x1F') {
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

  if (count != 9) {
    // TODO: same issue as in parse_pc_part_info()
    return 0;
  }

  return 1;
}

void serial_init(void) {
  
}

void serial_task(void) {
  // check serial string
  if (tud_cdc_available() && !inputStringReady) {
    // read datas
    uint32_t count = tud_cdc_read(inputString+inputStringLen, sizeof(inputString)-1-inputStringLen);
    inputStringLen += count;
    inputString[inputStringLen] = '\0';
    if (inputString[inputStringLen-2] == '\r' && inputString[inputStringLen-1] == '\n') {
      inputStringReady = 1;
    }
  }

  static uint64_t heartbeat_ms = 0;
  const uint64_t offset_heartbeat = 3000000;

  if (commstage == ErrorWait) {
    // TODO: have the device show an error screen and hold on it for some period of time.
    // screenstate = ErrorScreen;
    if (time_us_64() >= heartbeat_ms) {
      reset_state_data();
      return;
    }
  } else if (commstage == GreetHost) {
    if (inputStringReady && strncmp(inputString, "JB\x05", 3) == 0) {
      commstage = GreetDevice;
    }
    reset_input_string();
  } else if (commstage == GreetDevice) {
    tud_cdc_write("P001\r\n", 6);
    tud_cdc_write_flush();
    commstage = LinkConfirmHost;
    heartbeat_ms = time_us_64() + offset_heartbeat;
  } else if (commstage == LinkConfirmHost) {
    if (inputStringReady) {
      if (strncmp(inputString, "P\x06", 2) == 0) {
        commstage = LinkConfirmDevice;
      } else if (strncmp(inputString, "P\x15", 2) == 0) { // OR WE TIME OUT
        commstage = ErrorWait;
        heartbeat_ms = time_us_64() + offset_heartbeat;
      }
      reset_input_string();
    }
    if (time_us_64() >= heartbeat_ms) {
      reset_input_string();
      commstage = ErrorWait;
      heartbeat_ms = time_us_64() + offset_heartbeat;
    }
  } else if (commstage == LinkConfirmDevice) {
    tud_cdc_write("L\x06\r\n", 4);
    tud_cdc_write_flush();
    commstage = TransmitReady;
    heartbeat_ms = time_us_64() + offset_heartbeat;
  } else if (commstage == TransmitReady) {
    screenstate = ShowStats;
    // Check how long we've been waiting on a message
    if (!inputStringReady && time_us_64() >= heartbeat_ms) {
      commstage = ErrorWait;
      heartbeat_ms = time_us_64() + offset_heartbeat;
      return;
    }

    if (!inputStringReady) {
      return;
    }

    // Parse any incoming messages appropriately!
    if (inputString[0] == 'D') {
      // Device control
      if (inputString[1] == '\x11') {
        // PC Stats Control
        uint8_t parse_success = 0;
        if (inputString[2] == '\x30') {
          // "Unchanging" Info (CPU+GPU name, RAM capacity)
          parse_success = parse_pc_part_info();
        } else if (inputString[2] == '\x31') {
          // Stat Info (Temperature, Load%, etc.)
          parse_success = parse_pc_part_stats();
        }
        
        if (parse_success) {
          tud_cdc_write("D\x11\x06\r\n", 5);
        } else {
          tud_cdc_write("D\x11\x15\r\n", 5);
        }
        tud_cdc_write_flush();
        reset_input_string();
      } else if (inputString[1] == '\x12') {
        // RGB Control
        reset_input_string();
      }
    } else if (inputString[0] == 'H' && inputString[1] == '\x30') {
      // Heartbeat
      tud_cdc_write("H\x31\r\n", 4);
      tud_cdc_write_flush();
      heartbeat_ms = time_us_64() + offset_heartbeat;
      reset_input_string();
    }
  }
}
