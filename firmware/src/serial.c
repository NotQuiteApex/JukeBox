#include "serial.h"

#include <pico/bootrom.h>

#include <string.h>
#include <tusb.h>

extern volatile uint8_t bootsel_reset_jukebox;

extern ScreenState screenstate;
SerialStage commstage = GreetHost;

// The parts of comm stages (stage recieve and response)
int commstagepart = 0;

// The data string we use to store received data
char inputString[128] = "";
uint16_t inputStringLen = 0;
uint8_t inputStringReady = 0;

// Data we store!
// Data that we receive only at the time a connection is made.
char cpuName[48] = "";
char gpuName[48] = "";
char ramCount[10] = "";
// Data we constantly receive after a connection is made.
char cpuFreq[8] = "";
char cpuTemp[8] = "";
char cpuLoad[8] = "";
char ramUsed[8] = "";
char gpuTemp[8] = "";
char gpuCoreClock[10] = "";
char gpuCoreLoad[8]  = "";
char gpuVramClock[10] = "";
char gpuVramLoad[8]  = "";

uint32_t countermax = 0;

#define clear_string(s) \
	for (uint8_t i=0; i<sizeof(s); i++) { \
		s[i] = '\0'; \
	}

void reset_input_string(void) {
	clear_string(inputString);
	inputStringLen = 0;
	inputStringReady = 0;
}

void reset_state_data(void) {
	commstage = GreetHost;

	screenstate = WaitingConnection;

	reset_input_string();

	clear_string(cpuName);
	clear_string(gpuName);
	clear_string(ramCount);

	clear_string(cpuFreq);
	clear_string(cpuTemp);
	clear_string(cpuLoad);
	clear_string(ramUsed);
	clear_string(gpuTemp);
	clear_string(gpuCoreClock);
	clear_string(gpuCoreLoad);
	clear_string(gpuVramClock);
	clear_string(gpuVramLoad);
}

uint8_t parse_pc_part_info(void) {
	// process the names!
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
			size_t s = 0;
			char * i1 = inputString+idx1;
			if (count == 0) {
				s = MIN(sizeof(cpuName) - 1, idx2 - idx1);
				strncpy(cpuName, i1, s);
				cpuName[s] = '\0';
			} else if (count == 1) {
				s = MIN(sizeof(gpuName) - 1, idx2 - idx1);
				strncpy(gpuName, i1, s);
				gpuName[s] = '\0';
			} else if (count == 2) {
				s = MIN(sizeof(ramCount) - 1, idx2 - idx1);
				strncpy(ramCount, i1, s);
				ramCount[s] = '\0';
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

			size_t s = 0;
			char * i1 = inputString+idx1;
			if (count == 0) {
				s = MIN(sizeof(cpuFreq) - 1, idx2 - idx1);
				strncpy(cpuFreq, i1, s);
				cpuFreq[s] = '\0';
			} else if (count == 1) {
				s = MIN(sizeof(cpuTemp) - 1, idx2 - idx1);
				strncpy(cpuTemp, i1, s);
				cpuTemp[s] = '\0';
			} else if (count == 2) {
				s = MIN(sizeof(cpuLoad) - 1, idx2 - idx1);
				strncpy(cpuLoad, i1, s);
				cpuLoad[s] = '\0';
			} else if (count == 3) {
				s = MIN(sizeof(ramUsed) - 1, idx2 - idx1);
				strncpy(ramUsed, i1, s);
				ramUsed[s] = '\0';
			} else if (count == 4) {
				s = MIN(sizeof(gpuTemp) - 1, idx2 - idx1);
				strncpy(gpuTemp, i1, s);
				gpuTemp[s] = '\0';
			} else if (count == 5) {
				s = MIN(sizeof(gpuCoreClock) - 1, idx2 - idx1);
				strncpy(gpuCoreClock, i1, s);
				gpuCoreClock[s] = '\0';
			} else if (count == 6) {
				s = MIN(sizeof(gpuCoreLoad) - 1, idx2 - idx1);
				strncpy(gpuCoreLoad, i1, s);
				gpuCoreLoad[s] = '\0';
			} else if (count == 7) {
				s = MIN(sizeof(gpuVramClock) - 1, idx2 - idx1);
				strncpy(gpuVramClock, i1, s);
				gpuVramClock[s] = '\0';
			} else if (count == 8) {
				s = MIN(sizeof(gpuVramLoad) - 1, idx2 - idx1);
				strncpy(gpuVramLoad, i1, s);
				gpuVramLoad[s] = '\0';
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
	reset_state_data();
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
	const uint64_t offset_heartbeat = 1500000;

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
		} else if (inputString[0] == 'U') {
			if (inputString[1] == '\x30') {
				// Disconnect
				tud_cdc_write("\x04\x04\r\n", 4);
				tud_cdc_write_flush();
				reset_input_string();
				commstage = ErrorWait; // TODO: better handle serial disconnects
			} else if (inputString[1] == '\x31') {
				// Update
				tud_cdc_write("\x04\x04\r\n", 4);
				tud_cdc_write_flush();
				reset_input_string();
				sleep_ms(1000); // sleep to allow tinyusb to write cdc buffer (TODO: find better value)
				bootsel_reset_jukebox = 1;
				// TODO: deconstruct all the stuff on this core (screen, rgb, etc) and send signal to reset usb on main core
				// TODO: send a response to the desktop app before reset?
			}
		}
	}
}
