#pragma once

#include <avr/pgmspace.h>
#include <LUFA/Drivers/USB/USB.h>

#include "config.h"

#define KEYBOARD_EPADDR (ENDPOINT_DIR_IN | 1)
#define KEYBOARD_EPSIZE (8)

#ifdef SERIAL_COMMS
#define CDC_NOTIFICATION_EPADDR (ENDPOINT_DIR_IN | 2)
#define CDC_TX_EPADDR (ENDPOINT_DIR_IN | 3)
#define CDC_RX_EPADDR (ENDPOINT_DIR_OUT | 4)
#define CDC_NOTIFICATION_EPSIZE 8
#define CDC_TXRX_EPSIZE 16
#endif

typedef struct {
    USB_Descriptor_Configuration_Header_t Config;

    // Keyboard HID Interface
    USB_Descriptor_Interface_t HID_Interface;
    USB_HID_Descriptor_HID_t   HID_KeyboardHID;
    USB_Descriptor_Endpoint_t  HID_ReportINEndpoint;

#ifdef SERIAL_COMMS
    // Serial Interface (control)
    USB_Descriptor_Interface_t CDC_CCI_Interface;
    USB_CDC_Descriptor_FunctionalHeader_t CDC_Functional_Header;
    USB_CDC_Descriptor_FunctionalACM_t CDC_Functional_ACM;
    USB_CDC_Descriptor_FunctionalUnion_t CDC_Functional_Union;
    USB_Descriptor_Endpoint_t CDC_NotificationEndpoint;

    // Serial (Data)
    USB_Descriptor_Interface_t CDC_DCI_Interface;
    USB_Descriptor_Endpoint_t CDC_DataOutEndpoint;
    USB_Descriptor_Endpoint_t CDC_DataInEndpoint;
#endif
} USB_Descriptor_Configuration_t;

enum InterfaceDescriptors_t {
    INTERFACE_ID_Keyboard = 0, // Keyboard interface descriptor ID
    INTERFACE_ID_CDC_CCI = 1, // Serial control interface
    INTERFACE_ID_CDC_DCI = 2, // Serial data interface
};

enum StringDescriptors_t {
    STRING_ID_Language = 0,
    STRING_ID_Manufacturer = 1,
    STRING_ID_Product = 2,
};

uint16_t CALLBACK_USB_GetDescriptor(const uint16_t wValue, const uint16_t wIndex,
    const void** const DescriptorAddress, uint8_t* const DescriptorMemorySpace)
    ATTR_WARN_UNUSED_RESULT ATTR_NON_NULL_PTR_ARG(3);
