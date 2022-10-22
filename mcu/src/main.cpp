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

#include "keyboard.h"


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
	// Set all the keyboard pins as inputs with pullup (0 on DDR for input, 1 on PORTn for pullup)
	
	// set as inputs (COLUMNS, D2 D1 D4 C7)
	DDRD &= NIT(DD4) & NIT(DD2) & NIT(DD1);
	DDRC &= NIT(DD7);
	// pull em low
	PORTD &= NIT(PORT4) & NIT(PORT2) & NIT(PORT1);
	PORTC &= NIT(PORT7);

	// setting outputs (ROWS, D3 D0 C6)
	DDRD |= BIT(DD3) | BIT(DD0);
	DDRC |= BIT(DD6);
	// pull em low for now
	PORTD &= NIT(PORT3) & NIT(PORT0);
	PORTC &= NIT(PORT6);

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
	USB_Device_EnableSOFEvents();
}


void EVENT_USB_Device_ControlRequest() {
	HID_Device_ProcessControlRequest(&Keyboard_HID_Interface);
}


void EVENT_USB_Device_StartOfFrame() {
	HID_Device_MillisecondElapsed(&Keyboard_HID_Interface);
}


bool CALLBACK_HID_Device_CreateHIDReport(USB_ClassInfo_HID_Device_t* const HIDInterfaceInfo,
	uint8_t* const ReportID, const uint8_t ReportType, void* ReportData, uint16_t* const ReportSize)
{
	USB_KeyboardReport_Data_t* KeyboardReport = (USB_KeyboardReport_Data_t*)ReportData;

	uint8_t UsedKeyCodes = 0;

	// KEYS TO USE
	// HID_KEYBOARD_SC_F13-F24, _MEDIA_VOLUME_UP-DOWN (?)
	// PD4 - COL_3
	// PD3 - ROW_1
	// PD2 - COL_1
	// PD1 - COL_2
	// PD0 - ROW_2
	// PC7 - COL_4
	// PC6 - ROW_3

	// row is pulled high, column is checked, key press is added if check passed, and then row is pulled low
#define CHECK_KEY(INPORTX, INPIN, KEY) \
	if (INPORTX & BIT(INPIN) && UsedKeyCodes < MAX_NUMBER_OF_KEYS) \
		KeyboardReport->KeyCode[UsedKeyCodes++] = KEY; \

	PORTD |= BIT(PORT3);
	CHECK_KEY(PIND, PIN2, HID_KEYBOARD_SC_F13);
	CHECK_KEY(PIND, PIN1, HID_KEYBOARD_SC_F14);
	CHECK_KEY(PIND, PIN4, HID_KEYBOARD_SC_F15);
	CHECK_KEY(PINC, PIN7, HID_KEYBOARD_SC_F16);
	PORTD &= NIT(PORT3);

	PORTD |= BIT(PORT0);
	CHECK_KEY(PIND, PIN2, HID_KEYBOARD_SC_F17);
	CHECK_KEY(PIND, PIN1, HID_KEYBOARD_SC_F18);
	CHECK_KEY(PIND, PIN4, HID_KEYBOARD_SC_F19);
	CHECK_KEY(PINC, PIN7, HID_KEYBOARD_SC_F20);
	PORTD &= NIT(PORT0);

	PORTC |= BIT(PORT6);
	CHECK_KEY(PIND, PIN2, HID_KEYBOARD_SC_F21);
	CHECK_KEY(PIND, PIN1, HID_KEYBOARD_SC_F22);
	CHECK_KEY(PIND, PIN4, HID_KEYBOARD_SC_F23);
	CHECK_KEY(PINC, PIN7, HID_KEYBOARD_SC_F24);
	PORTC &= NIT(PORT6);

#undef CHECK_KEY

	*ReportSize = sizeof(USB_KeyboardReport_Data_t);
	return false;
}


void CALLBACK_HID_Device_ProcessHIDReport(USB_ClassInfo_HID_Device_t* const HIDInterfaceInfo,
	const uint8_t ReportID, const uint8_t ReportType, const void* ReportData, const uint16_t ReportSize)
{
	// Nobody here but us chickens.
}
