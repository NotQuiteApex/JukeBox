// Based on code from the LUFA project here:
// https://github.com/abcminiuser/lufa/tree/master/Demos/Device/ClassDriver/Keyboard
// https://github.com/abcminiuser/lufa/tree/master/Demos/Device/ClassDriver/VirtualSerial

// PINOUT
// - KEYS FOR KEYBOARD -
// PD4 - COL_3
// PD3 - ROW_1
// PD2 - COL_1
// PD1 - COL_2
// PD0 - ROW_2
// PC7 - COL_4
// PC6 - ROW_3
// - CONNECTIONS FOR SCREEN -
// PB7 - CS (chip select)
// PB6 - DC
// PB5 - RST (reset)
// PB4 - BL
// PB2 - DIN (mosi)
// PB1 - CLK (sck)

#define BIT(x) (1<<x)
#define NIT(x) (~(1<<x))

#define _NOP() do { __asm__ __volatile__ ("nop"); } while (0)

#include "keyboard.h"


// debounce utils
// timer max should be adjusted if doing 8mhz or 16mhz clock
#define DEBOUNCE_TIMER_MAX 100
bool states[3][4] = {
	{0, 0, 0, 0},
	{0, 0, 0, 0},
	{0, 0, 0, 0}
};
uint8_t debounce[3][4] = {
	{0, 0, 0, 0},
	{0, 0, 0, 0},
	{0, 0, 0, 0}
};


static uint8_t PrevKeyboardHIDReportBuffer[sizeof(USB_KeyboardReport_Data_t)];

USB_ClassInfo_HID_Device_t Keyboard_HID_Interface = {
	.Config = {
		.InterfaceNumber = INTERFACE_ID_Keyboard,
		.ReportINEndpoint = {
			.Address = KEYBOARD_EPADDR, .Size = KEYBOARD_EPSIZE,
			.Type = 0, .Banks = 1,
		},
		.PrevReportINBuffer = PrevKeyboardHIDReportBuffer,
		.PrevReportINBufferSize = sizeof(PrevKeyboardHIDReportBuffer),
	}
};


#ifdef SERIAL_COMMS
#include <stdio.h>
static FILE USBSerialStream;
USB_ClassInfo_CDC_Device_t VirtualSerial_CDC_Interface = {
	.Config = {
		.ControlInterfaceNumber = INTERFACE_ID_CDC_CCI,
		.DataINEndpoint = {
			.Address = CDC_TX_EPADDR, .Size = CDC_TXRX_EPSIZE,
			.Type = 0, .Banks = 1,
		},
		.DataOUTEndpoint = {
			.Address = CDC_RX_EPADDR, .Size = CDC_TXRX_EPSIZE,
			.Type = 0, .Banks = 1,
		},
		.NotificationEndpoint = {
			.Address = CDC_NOTIFICATION_EPADDR, .Size = CDC_NOTIFICATION_EPSIZE,
			.Type = 0, .Banks = 1,
		},
	},
};
#endif


int main() {
	// setup pins
	// setting inputs and outputs (COLUMNS, D2 D1 D4 C7; ROWS, D3 D0 C6)
	DDRD = (BIT(DD3) | BIT(DD0)) & (NIT(DD4) & NIT(DD2) & NIT(DD1));
	DDRC = BIT(DD6) & NIT(DD7);
	// pull em low, all low (3, 0, and 6 are rows)
	PORTD = NIT(PORT4) & NIT(PORT2) & NIT(PORT1) & NIT(PORT3) & NIT(PORT0);
	PORTC = NIT(PORT7) & NIT(PORT6);

	// disable timers and clock divisers
	wdt_disable();
	clock_prescale_set(clock_div_1);
	GlobalInterruptEnable();

	// setup usb interfaces
	//Joystick_Init();
	//Buttons_Init();
	USB_Init(USE_STATIC_OPTIONS);

#ifdef SERIAL_COMMS
	CDC_Device_CreateStream(&VirtualSerial_CDC_Interface, &USBSerialStream);
#endif

	while (true) {
#ifdef SERIAL_COMMS
		// get stream size, malloc, and append to storage
		uint16_t size = CDC_Device_BytesReceived(&VirtualSerial_CDC_Interface);
		if (size > 0) {
			fprintf(&USBSerialStream, "len:%d",size);
			for (uint16_t i=0; i<size; i++) {
				uint16_t c = CDC_Device_ReceiveByte(&VirtualSerial_CDC_Interface);
				fprintf(&USBSerialStream, "char:%d(%c)", c, c);
			}
		}

		// Handle serial events
		CDC_Device_USBTask(&VirtualSerial_CDC_Interface);
#endif

		// Handle keyboard events
		HID_Device_USBTask(&Keyboard_HID_Interface);
		// Handle general USB stuff
		USB_USBTask();
	}

	return 0;
}


void EVENT_USB_Device_Connect() {
	// When a USB connection is established.
	// Nothing yet!
}


void EVENT_USB_Device_Disconnect() {
	// When a USB connection is lost.
	// Nothing yet!
}


void EVENT_USB_Device_ConfigurationChanged() {
	bool success = true;
	success &= HID_Device_ConfigureEndpoints(&Keyboard_HID_Interface);
	// USB_Device_EnableSOFEvents();
}


void EVENT_USB_Device_ControlRequest() {
	HID_Device_ProcessControlRequest(&Keyboard_HID_Interface);
}


// void EVENT_USB_Device_StartOfFrame() {
// 	HID_Device_MillisecondElapsed(&Keyboard_HID_Interface);
// }


bool CALLBACK_HID_Device_CreateHIDReport(USB_ClassInfo_HID_Device_t* const HIDInterfaceInfo,
	uint8_t* const ReportID, const uint8_t ReportType, void* ReportData, uint16_t* const ReportSize)
{
	USB_KeyboardReport_Data_t* KeyboardReport = (USB_KeyboardReport_Data_t*)ReportData;

	uint8_t UsedKeyCodes = 0;
	
	static volatile uint8_t * ports[] = {&PORTD, &PORTD, &PORTC};
	static uint8_t opins[] = {BIT(PORT3), BIT(PORT0), BIT(PORT6)};
	static volatile uint8_t * pins[] = {&PIND, &PIND, &PIND, &PINC};
	static uint8_t ipins[] = {BIT(PIN2), BIT(PIN1), BIT(PIN4), BIT(PIN7)};

	// row is pulled high, column is checked, key press is added if check passed, and then row is pulled low
	uint8_t k = 0;
	for (size_t i=0; i<3; i++) {
		*(ports[i]) = opins[i];
		_NOP(); _NOP(); _NOP();
		for (size_t j=0; j<4; j++) {
			// check on all states
			bool state = (*(pins[j]) & ipins[j]) != 0;
			bool prevstate = states[i][j];
			if (prevstate != state) {
				// state is unstable, reset timer
				debounce[i][j] = 0;
			} else {
				// state is stable, set debounced state
				debounce[i][j]++;
				if (debounce[i][j] >= DEBOUNCE_TIMER_MAX) {
					debounce[i][j] = 0;
					states[i][j] = state;
				}
			}

			if (UsedKeyCodes >= MAX_NUMBER_OF_KEYS) {
				continue;
			}

			if (states[i][j]) {
				KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_F13 + k;
			}
			k++;
		}
	}

	*(ports[0]) = 0;
	*(ports[2]) = 0;

	*ReportSize = sizeof(USB_KeyboardReport_Data_t);
	return false;
}


void CALLBACK_HID_Device_ProcessHIDReport(USB_ClassInfo_HID_Device_t* const HIDInterfaceInfo,
	const uint8_t ReportID, const uint8_t ReportType, const void* ReportData, const uint16_t ReportSize)
{
	// Nobody here but us chickens.
}
