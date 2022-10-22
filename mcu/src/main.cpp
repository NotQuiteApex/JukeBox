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


#include "keyboard.h"


#define BIT(x) (1<<x)
#define NIT(x) (~(1<<x))


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

static FILE USBSerialStream;


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
	// pull em high
	PORTD |= BIT(PORT3) | BIT(PORT0);
	PORTC |= BIT(PORT6);

	// disable timers and clock divisers
	wdt_disable();
	clock_prescale_set(clock_div_1);

	// setup usb interfaces
	//Joystick_Init();
	//Buttons_Init();
	USB_Init(USE_STATIC_OPTIONS);
	CDC_Device_CreateStream(&VirtualSerial_CDC_Interface, &USBSerialStream);

	GlobalInterruptEnable();

	while (true) {
		HID_Device_USBTask(&Keyboard_HID_Interface);
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

	//uint8_t JoyStatus_LCL    = Joystick_GetStatus();
	//uint8_t ButtonStatus_LCL = Buttons_GetStatus();

	uint8_t UsedKeyCodes = 0;

	// KEYS TO USE
	// HID_KEYBOARD_SC_F13-F24, _MEDIA_VOLUME_UP-DOWN
	// PD4 - COL_3
	// PD3 - ROW_1
	// PD2 - COL_1
	// PD1 - COL_2
	// PD0 - ROW_2
	// PC7 - COL_4
	// PC6 - ROW_3

	uint8_t pind = PIND;
	uint8_t pinc = PINC;

	uint8_t COL_1 = pind & BIT(PIN2);
	uint8_t COL_2 = pind & BIT(PIN1);
	uint8_t COL_3 = pind & BIT(PIN4);
	uint8_t COL_4 = pinc & BIT(PIN7);

	uint8_t ROW_1 = pind & BIT(PIN3);
	uint8_t ROW_2 = pind & NIT(PIN0);
	uint8_t ROW_3 = pinc & NIT(PIN6);

	if (ROW_1 && COL_1)
		KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_F13;

	*ReportSize = sizeof(USB_KeyboardReport_Data_t);
	return false;
}


void CALLBACK_HID_Device_ProcessHIDReport(USB_ClassInfo_HID_Device_t* const HIDInterfaceInfo,
	const uint8_t ReportID, const uint8_t ReportType, const void* ReportData, const uint16_t ReportSize)
{
	// uint8_t  LEDMask   = LEDS_NO_LEDS;
	// uint8_t* LEDReport = (uint8_t*)ReportData;

	// if (*LEDReport & HID_KEYBOARD_LED_NUMLOCK)
	//   LEDMask |= LEDS_LED1;

	// if (*LEDReport & HID_KEYBOARD_LED_CAPSLOCK)
	//   LEDMask |= LEDS_LED3;

	// if (*LEDReport & HID_KEYBOARD_LED_SCROLLLOCK)
	//   LEDMask |= LEDS_LED4;

	// LEDs_SetAllLEDs(LEDMask);
}
