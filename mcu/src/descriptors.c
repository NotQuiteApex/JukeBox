#include "descriptors.h"

// TODO: add descriptors for serial device communication
// https://groups.google.com/g/lufa-support/c/axkQqopa_t4

const USB_Descriptor_HIDReport_Datatype_t PROGMEM KeyboardReport[] = {
	HID_DESCRIPTOR_KEYBOARD(6) // Max of 6 simultaneous keys
};

const USB_Descriptor_Device_t PROGMEM DeviceDescriptor = {
	.Header = { .Size = sizeof(USB_Descriptor_Device_t), .Type = DTYPE_Device },

	.USBSpecification = VERSION_BCD(1,1,0),
	.Class    = USB_CSCP_NoDeviceClass | CDC_CSCP_CDCClass,
	.SubClass = USB_CSCP_NoDeviceSubclass | CDC_CSCP_NoSpecificSubclass,
	.Protocol = USB_CSCP_NoDeviceProtocol | CDC_CSCP_NoSpecificProtocol,

	.Endpoint0Size = FIXED_CONTROL_ENDPOINT_SIZE,

	.VendorID      = USB_DEVICE_VID,
	.ProductID     = USB_DEVICE_PID,
	.ReleaseNumber = VERSION_BCD(0,0,1),

	.ManufacturerStrIndex = STRING_ID_Manufacturer,
	.ProductStrIndex = STRING_ID_Product,
	.SerialNumStrIndex = NO_DESCRIPTOR | USE_INTERNAL_SERIAL,

	.NumberOfConfigurations = FIXED_NUM_CONFIGURATIONS,
};

const USB_Descriptor_Configuration_t PROGMEM ConfigurationDescriptor = {
	.Config = {
		.Header = { .Size = sizeof(USB_Descriptor_Configuration_Header_t), .Type = DTYPE_Configuration },
		.TotalConfigurationSize = sizeof(USB_Descriptor_Configuration_t),
		.TotalInterfaces = 3,
		.ConfigurationNumber = 1,
		.ConfigurationStrIndex = NO_DESCRIPTOR,
		.ConfigAttributes = USB_CONFIG_ATTR_RESERVED,
		.MaxPowerConsumption = USB_CONFIG_POWER_MA(200),
	},

	// Keyboard Interface
	.HID_Interface = {
		.Header = { .Size = sizeof(USB_Descriptor_Interface_t), .Type = DTYPE_Interface },
		.InterfaceNumber = INTERFACE_ID_Keyboard,
		.AlternateSetting = 0x0,
		.TotalEndpoints = 1,
		.Class = HID_CSCP_HIDClass,
		.SubClass = HID_CSCP_BootSubclass,
		.Protocol = HID_CSCP_KeyboardBootProtocol,
		.InterfaceStrIndex = NO_DESCRIPTOR,
	},
	.HID_KeyboardHID = {
		.Header = { .Size = sizeof(USB_HID_Descriptor_HID_t), .Type = HID_DTYPE_HID },
		.HIDSpec = VERSION_BCD(1,1,1),
		.CountryCode = 0x0,
		.TotalReportDescriptors = 1,
		.HIDReportType = HID_DTYPE_Report,
		.HIDReportLength = sizeof(KeyboardReport),
	},
	.HID_ReportINEndpoint = {
		.Header = { .Size = sizeof(USB_Descriptor_Endpoint_t), .Type = DTYPE_Endpoint },
		.EndpointAddress = KEYBOARD_EPADDR,
		.Attributes = (EP_TYPE_INTERRUPT | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA),
		.EndpointSize = KEYBOARD_EPSIZE,
		.PollingIntervalMS = 0x05,
	},

	// Serial Interface
	.CDC_CCI_Interface = {
		.Header = { .Size = sizeof(USB_Descriptor_Interface_t), .Type = DTYPE_Interface },
		.InterfaceNumber = INTERFACE_ID_CDC_CCI,
		.AlternateSetting = 0,
		.TotalEndpoints = 1,
		.Class = CDC_CSCP_CDCClass,
		.SubClass = CDC_CSCP_ACMSubclass,
		.Protocol = CDC_CSCP_ATCommandProtocol,
		.InterfaceStrIndex = NO_DESCRIPTOR,
	},
	.CDC_Functional_Header = {
		.Header = { .Size = sizeof(USB_CDC_Descriptor_FunctionalHeader_t), .Type = CDC_DTYPE_CSInterface },
		.Subtype = CDC_DSUBTYPE_CSInterface_Header,
		.CDCSpecification = VERSION_BCD(1,1,0),
	},
	.CDC_Functional_ACM = {
		.Header = { .Size = sizeof(USB_CDC_Descriptor_FunctionalACM_t), .Type = CDC_DTYPE_CSInterface },
		.Subtype = CDC_DSUBTYPE_CSInterface_ACM,
		.Capabilities = 0x06,
	},
	.CDC_Functional_Union = {
		.Header = { .Size = sizeof(USB_CDC_Descriptor_FunctionalUnion_t), .Type = CDC_DTYPE_CSInterface },
		.Subtype = CDC_DSUBTYPE_CSInterface_Union,
		.MasterInterfaceNumber = INTERFACE_ID_CDC_CCI,
		.SlaveInterfaceNumber = INTERFACE_ID_CDC_DCI,
	},
	.CDC_NotificationEndpoint = {
		.Header = { .Size = sizeof(USB_Descriptor_Endpoint_t), .Type = DTYPE_Endpoint },
		.EndpointAddress = CDC_NOTIFICATION_EPADDR,
		.Attributes = (EP_TYPE_INTERRUPT | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA),
		.EndpointSize = CDC_NOTIFICATION_EPSIZE,
		.PollingIntervalMS = 0xFF,
	},
	.CDC_DCI_Interface = {
		.Header = { .Size = sizeof(USB_Descriptor_Interface_t), .Type = DTYPE_Interface },
		.InterfaceNumber = INTERFACE_ID_CDC_DCI,
		.AlternateSetting = 0,
		.TotalEndpoints = 2,
		.Class = CDC_CSCP_CDCDataClass,
		.SubClass = CDC_CSCP_NoDataSubclass,
		.Protocol = CDC_CSCP_NoDataProtocol,
		.InterfaceStrIndex = NO_DESCRIPTOR,
	},
	.CDC_DataOutEndpoint = {
		.Header = { .Size = sizeof(USB_Descriptor_Endpoint_t), .Type = DTYPE_Endpoint },
		.EndpointAddress = CDC_RX_EPADDR,
		.Attributes = (EP_TYPE_BULK | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA),
		.EndpointSize = CDC_TXRX_EPSIZE,
		.PollingIntervalMS = 0x05,
	},
	.CDC_DataOutEndpoint = {
		.Header = { .Size = sizeof(USB_Descriptor_Endpoint_t), .Type = DTYPE_Endpoint },
		.EndpointAddress = CDC_TX_EPADDR,
		.Attributes = (EP_TYPE_BULK | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA),
		.EndpointSize = CDC_TXRX_EPSIZE,
		.PollingIntervalMS = 0x05,
	},
};

const USB_Descriptor_String_t PROGMEM LanguageString = USB_STRING_DESCRIPTOR_ARRAY(LANGUAGE_ID_ENG);
const USB_Descriptor_String_t PROGMEM ManufacturerString = USB_STRING_DESCRIPTOR(USB_DEVICE_MANUFACTURER);
const USB_Descriptor_String_t PROGMEM ProductString = USB_STRING_DESCRIPTOR(USB_DEVICE_PRODUCT);

uint16_t CALLBACK_USB_GetDescriptor(const uint16_t wValue, const uint16_t wIndex,
	const void** const DescriptorAddress, uint8_t* const DescriptorMemorySpace)
{
	const uint8_t  DescriptorType   = (wValue >> 8);
	const uint8_t  DescriptorNumber = (wValue & 0xFF);

	const void* Address = NULL;
	uint16_t    Size    = NO_DESCRIPTOR;

	switch (DescriptorType)
	{
		case DTYPE_Device:
			Address = &DeviceDescriptor;
			Size    = sizeof(USB_Descriptor_Device_t);
			break;
		case DTYPE_Configuration:
			Address = &ConfigurationDescriptor;
			Size    = sizeof(USB_Descriptor_Configuration_t);
			break;
		case DTYPE_String:
			switch (DescriptorNumber)
			{
				case STRING_ID_Language:
					Address = &LanguageString;
					Size    = pgm_read_byte(&LanguageString.Header.Size);
					break;
				case STRING_ID_Manufacturer:
					Address = &ManufacturerString;
					Size    = pgm_read_byte(&ManufacturerString.Header.Size);
					break;
				case STRING_ID_Product:
					Address = &ProductString;
					Size    = pgm_read_byte(&ProductString.Header.Size);
					break;
			}

			break;
		case HID_DTYPE_HID:
			Address = &ConfigurationDescriptor.HID_KeyboardHID;
			Size    = sizeof(USB_HID_Descriptor_HID_t);
			break;
		case HID_DTYPE_Report:
			Address = &KeyboardReport;
			Size    = sizeof(KeyboardReport);
			break;
	}

	*DescriptorAddress = Address;
	return Size;
}
