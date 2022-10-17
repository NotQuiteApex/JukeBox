// Based on code from the LUFA project here:
// https://github.com/abcminiuser/lufa/tree/master/Demos/Device/ClassDriver/Keyboard

// PINOUT
// - KEYS FOR KEYBOARD -
// PD4 - COL_3
// PD3 - ROW_1
// PD2 - COL_1
// PD1 - COL_2
// PD0 - ROW_2
// PC7 - COL_4
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
			.Address = KEYBOARD_EPADDR,
			.Size = KEYBOARD_EPSIZE,
			.Type = 0,
			.Banks = 1,
		},
		.PrevReportINBuffer = PrevKeyboardHIDReportBuffer,
		.PrevReportINBufferSize = sizeof(PrevKeyboardHIDReportBuffer),
	}
};


int main() {
	// setup pins
	// Set all the keyboard pins as inputs with pullup (0 on DDR for input, 1 on PORTn for pullup)
	DDRB &= NIT(DD4) & NIT(DD3) & NIT(DD2) & NIT(DD1) & NIT(DD0);
	DDRC &= NIT(DD7);
	PORTB |= BIT(PORT4) | BIT(PORT3) | BIT(PORT2) | BIT(PORT1) | BIT(PORT0);
	PORTC |= BIT(PORT7);

	// disable timers and clock divisers
	wdt_disable();
	clock_prescale_set(clock_div_1);

	// setup usb interfaces
	Joystick_Init();
	Buttons_Init();
	USB_Init(USE_STATIC_OPTIONS);

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

	uint8_t JoyStatus_LCL    = Joystick_GetStatus();
	uint8_t ButtonStatus_LCL = Buttons_GetStatus();

	uint8_t UsedKeyCodes = 0;

	// if (JoyStatus_LCL & JOY_UP)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_A;
	// else if (JoyStatus_LCL & JOY_DOWN)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_B;

	// if (JoyStatus_LCL & JOY_LEFT)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_C;
	// else if (JoyStatus_LCL & JOY_RIGHT)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_D;

	// if (JoyStatus_LCL & JOY_PRESS)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_E;

	// if (ButtonStatus_LCL & BUTTONS_BUTTON1)
	//   KeyboardReport->KeyCode[UsedKeyCodes++] = HID_KEYBOARD_SC_F;

	// if (UsedKeyCodes)
	//   KeyboardReport->Modifier = HID_KEYBOARD_MODIFIER_LEFTSHIFT;

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
