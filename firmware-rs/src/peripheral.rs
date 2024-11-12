// Peripheral info

use rp_pico::hal::usb::UsbBus;
use usbd_serial::SerialPort;

pub const PERIPHERAL_ID_KEYBOARD: u8 = 0b1000_0000;
pub const PERIPHERAL_ID_KNOBS_1: u8 = 0b1000_0010;
pub const PERIPHERAL_ID_KNOBS_2: u8 = 0b1000_0011;
pub const PERIPHERAL_ID_PEDAL_1: u8 = 0b1000_0101;
pub const PERIPHERAL_ID_PEDAL_2: u8 = 0b1000_0110;
pub const PERIPHERAL_ID_PEDAL_3: u8 = 0b1000_0111;

#[derive(PartialEq, Clone, Copy)]
pub enum Connection {
    NotConnected,
    Connected,
}
impl Connection {
    const fn default() -> Self {
        Connection::NotConnected
    }

    pub fn connected(&self) -> bool {
        match self {
            Connection::NotConnected => false,
            Connection::Connected => true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct JBPeripherals {
    pub keyboard: Connection,
    pub knobs1: Connection,
    pub knobs2: Connection,
    pub pedal1: Connection,
    pub pedal2: Connection,
    pub pedal3: Connection,
}
impl JBPeripherals {
    pub const fn default() -> Self {
        JBPeripherals {
            keyboard: Connection::default(),
            knobs1: Connection::default(),
            knobs2: Connection::default(),
            pedal1: Connection::default(),
            pedal2: Connection::default(),
            pedal3: Connection::default(),
        }
    }

    pub fn write_report(self, serial: &mut SerialPort<UsbBus>) {
        if self.keyboard.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_KEYBOARD]);
        }
        if self.knobs1.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_KNOBS_1]);
        }
        if self.knobs2.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_KNOBS_2]);
        }
        if self.pedal1.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_PEDAL_1]);
        }
        if self.pedal2.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_PEDAL_2]);
        }
        if self.pedal3.connected() {
            let _ = serial.write(&[PERIPHERAL_ID_PEDAL_3]);
        }
    }
}

#[derive(Clone, Copy)]
pub enum SwitchPosition {
    Up,
    Down,
}
impl SwitchPosition {
    pub const fn default() -> Self {
        SwitchPosition::Up
    }

    pub fn encode(self, pos: u8) -> u8 {
        match self {
            SwitchPosition::Up => 0,
            SwitchPosition::Down => 1 << pos,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyboardInputs {
    pub key1: SwitchPosition,
    pub key2: SwitchPosition,
    pub key3: SwitchPosition,
    pub key4: SwitchPosition,
    pub key5: SwitchPosition,
    pub key6: SwitchPosition,
    pub key7: SwitchPosition,
    pub key8: SwitchPosition,
    pub key9: SwitchPosition,
    pub key10: SwitchPosition,
    pub key11: SwitchPosition,
    pub key12: SwitchPosition,
    pub key13: SwitchPosition,
    pub key14: SwitchPosition,
    pub key15: SwitchPosition,
    pub key16: SwitchPosition,
}
impl KeyboardInputs {
    pub const fn default() -> Self {
        KeyboardInputs {
            key1: SwitchPosition::default(),
            key2: SwitchPosition::default(),
            key3: SwitchPosition::default(),
            key4: SwitchPosition::default(),
            key5: SwitchPosition::default(),
            key6: SwitchPosition::default(),
            key7: SwitchPosition::default(),
            key8: SwitchPosition::default(),
            key9: SwitchPosition::default(),
            key10: SwitchPosition::default(),
            key11: SwitchPosition::default(),
            key12: SwitchPosition::default(),
            key13: SwitchPosition::default(),
            key14: SwitchPosition::default(),
            key15: SwitchPosition::default(),
            key16: SwitchPosition::default(),
        }
    }

    pub fn encode(self) -> [u8; 3] {
        let w2 = self.key16.encode(7)
            | self.key15.encode(6)
            | self.key14.encode(5)
            | self.key13.encode(4)
            | self.key12.encode(3)
            | self.key11.encode(2)
            | self.key10.encode(1)
            | self.key9.encode(0);
        let w1 = self.key8.encode(7)
            | self.key7.encode(6)
            | self.key6.encode(5)
            | self.key5.encode(4)
            | self.key4.encode(3)
            | self.key3.encode(2)
            | self.key2.encode(1)
            | self.key1.encode(0);
        [PERIPHERAL_ID_KEYBOARD, w2, w1]
    }
}

#[derive(Clone, Copy)]
pub enum KnobDirection {
    None,
    Clockwise,
    CounterClockwise,
}
impl KnobDirection {
    pub const fn default() -> Self {
        KnobDirection::None
    }

    pub fn encode(self, pos: u8) -> u8 {
        match self {
            KnobDirection::None => 0,
            KnobDirection::Clockwise => 0b0000_0001 << pos,
            KnobDirection::CounterClockwise => 0b0000_0010 << pos,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KnobInputs {
    pub knob_left_switch: SwitchPosition,
    pub knob_left_direction: KnobDirection,
    pub knob_right_switch: SwitchPosition,
    pub knob_right_direction: KnobDirection,
}
impl KnobInputs {
    pub const fn default() -> Self {
        KnobInputs {
            knob_left_switch: SwitchPosition::default(),
            knob_left_direction: KnobDirection::default(),
            knob_right_switch: SwitchPosition::default(),
            knob_right_direction: KnobDirection::default(),
        }
    }

    fn encode(self, id: u8) -> [u8; 2] {
        let w = self.knob_left_switch.encode(5)
            | self.knob_left_direction.encode(3)
            | self.knob_right_switch.encode(2)
            | self.knob_right_direction.encode(0);

        [id, w]
    }

    pub fn encode_1(self) -> [u8; 2] {
        self.encode(PERIPHERAL_ID_KNOBS_1)
    }

    pub fn encode_2(self) -> [u8; 2] {
        self.encode(PERIPHERAL_ID_KNOBS_2)
    }
}

#[derive(Clone, Copy)]
pub struct PedalInputs {
    pub pedal_left: SwitchPosition,
    pub pedal_middle: SwitchPosition,
    pub pedal_right: SwitchPosition,
}
impl PedalInputs {
    pub const fn default() -> Self {
        PedalInputs {
            pedal_left: SwitchPosition::default(),
            pedal_middle: SwitchPosition::default(),
            pedal_right: SwitchPosition::default(),
        }
    }

    fn encode(self, id: u8) -> [u8; 2] {
        [
            id,
            self.pedal_left.encode(2) | self.pedal_middle.encode(1) | self.pedal_right.encode(0),
        ]
    }

    pub fn encode_1(self) -> [u8; 2] {
        self.encode(PERIPHERAL_ID_PEDAL_1)
    }

    pub fn encode_2(self) -> [u8; 2] {
        self.encode(PERIPHERAL_ID_PEDAL_2)
    }

    pub fn encode_3(self) -> [u8; 2] {
        self.encode(PERIPHERAL_ID_PEDAL_3)
    }
}

#[derive(Clone, Copy)]
pub struct JBPeripheralInputs {
    pub keyboard: KeyboardInputs,
    pub knobs1: KnobInputs,
    pub knobs2: KnobInputs,
    pub pedal1: PedalInputs,
    pub pedal2: PedalInputs,
    pub pedal3: PedalInputs,
}
impl JBPeripheralInputs {
    pub const fn default() -> Self {
        JBPeripheralInputs {
            keyboard: KeyboardInputs::default(),
            knobs1: KnobInputs::default(),
            knobs2: KnobInputs::default(),
            pedal1: PedalInputs::default(),
            pedal2: PedalInputs::default(),
            pedal3: PedalInputs::default(),
        }
    }

    pub fn write_report(self, peripherals: JBPeripherals, serial: &mut SerialPort<UsbBus>) {
        if peripherals.keyboard.connected() {
            let _ = serial.write(&self.keyboard.encode());
        }
        if peripherals.knobs1.connected() {
            let _ = serial.write(&self.knobs1.encode_1());
        }
        if peripherals.knobs2.connected() {
            let _ = serial.write(&self.knobs2.encode_2());
        }
        if peripherals.pedal1.connected() {
            let _ = serial.write(&self.pedal1.encode_1());
        }
        if peripherals.pedal2.connected() {
            let _ = serial.write(&self.pedal2.encode_2());
        }
        if peripherals.pedal3.connected() {
            let _ = serial.write(&self.pedal3.encode_3());
        }
    }
}
