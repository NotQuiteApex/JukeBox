// All the utilities for the input system

use bitmatch::bitmatch;

pub const IDENT_UNKNOWN_INPUT: u8 = b'?';
pub const IDENT_KEY_INPUT: u8 = b'K';
pub const IDENT_KNOB_INPUT: u8 = b'O';
pub const IDENT_PEDAL_INPUT: u8 = b'P';

#[derive(PartialEq, Clone, Copy)]
pub enum Connection {
    NotConnected(bool), // false - lost connection, true - clean disconnect
    Connected,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SwitchPosition {
    Up,
    Down,
}
impl SwitchPosition {
    pub const fn default() -> Self {
        Self::Up
    }

    pub fn is_down(self) -> bool {
        match self {
            SwitchPosition::Up => false,
            SwitchPosition::Down => true,
        }
    }
}
impl Into<SwitchPosition> for bool {
    fn into(self) -> SwitchPosition {
        match self {
            true => SwitchPosition::Down,
            false => SwitchPosition::Up,
        }
    }
}
impl Into<SwitchPosition> for u8 {
    fn into(self) -> SwitchPosition {
        match self {
            1 => SwitchPosition::Down,
            _ => SwitchPosition::Up,
        }
    }
}
impl Into<u8> for SwitchPosition {
    fn into(self) -> u8 {
        match self {
            Self::Down => 1,
            Self::Up => 0,
        }
    }
}
impl Into<SwitchPosition> for u16 {
    fn into(self) -> SwitchPosition {
        match self {
            1 => SwitchPosition::Down,
            _ => SwitchPosition::Up,
        }
    }
}
impl Into<bool> for SwitchPosition {
    fn into(self) -> bool {
        match self {
            Self::Down => true,
            Self::Up => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyInputs {
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
impl KeyInputs {
    pub const fn default() -> Self {
        KeyInputs {
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

    #[bitmatch]
    pub fn encode(self) -> [u8; 3] {
        let p: u8 = self.key16.into();
        let o: u8 = self.key15.into();
        let n: u8 = self.key14.into();
        let m: u8 = self.key13.into();
        let l: u8 = self.key12.into();
        let k: u8 = self.key11.into();
        let j: u8 = self.key10.into();
        let i: u8 = self.key9.into();
        let h: u8 = self.key8.into();
        let g: u8 = self.key7.into();
        let f: u8 = self.key6.into();
        let e: u8 = self.key5.into();
        let d: u8 = self.key4.into();
        let c: u8 = self.key3.into();
        let b: u8 = self.key2.into();
        let a: u8 = self.key1.into();

        [IDENT_KEY_INPUT, bitpack!("ponmlkji"), bitpack!("hgfedcba")]
    }

    #[bitmatch]
    pub fn decode(b: &[u8]) -> Result<Self, ()> {
        if b.len() != 3
            || *b.get(0).unwrap_or(&b'\0') != IDENT_KEY_INPUT
            || b.get(1).is_none()
            || b.get(2).is_none()
        {
            return Err(());
        }

        let w1 = *b.get(1).unwrap();
        let w2 = *b.get(2).unwrap();
        let w: u16 = (w1 as u16) << 8 | w2 as u16;

        #[bitmatch]
        match w {
            "ponmlkjihgfedcba" => Ok(KeyInputs {
                key1: a.into(),
                key2: b.into(),
                key3: c.into(),
                key4: d.into(),
                key5: e.into(),
                key6: f.into(),
                key7: g.into(),
                key8: h.into(),
                key9: i.into(),
                key10: j.into(),
                key11: k.into(),
                key12: l.into(),
                key13: m.into(),
                key14: n.into(),
                key15: o.into(),
                key16: p.into(),
            }),
            _ => Err(()),
        }
    }
}
impl Into<KeyInputs> for [bool; 16] {
    fn into(self) -> KeyInputs {
        KeyInputs {
            key1: self[0].into(),
            key2: self[1].into(),
            key3: self[2].into(),
            key4: self[3].into(),
            key5: self[4].into(),
            key6: self[5].into(),
            key7: self[6].into(),
            key8: self[7].into(),
            key9: self[8].into(),
            key10: self[9].into(),
            key11: self[10].into(),
            key12: self[11].into(),
            key13: self[12].into(),
            key14: self[13].into(),
            key15: self[14].into(),
            key16: self[15].into(),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum KnobDirection {
    None,
    Clockwise,
    CounterClockwise,
}
impl KnobDirection {
    pub const fn default() -> Self {
        KnobDirection::None
    }

    pub fn is_clockwise(self) -> bool {
        match self {
            Self::Clockwise => true,
            _ => false,
        }
    }

    pub fn is_counter_clockwise(self) -> bool {
        match self {
            Self::CounterClockwise => true,
            _ => false,
        }
    }
}
impl Into<KnobDirection> for u8 {
    fn into(self) -> KnobDirection {
        match self {
            0b01 => KnobDirection::Clockwise,
            0b10 => KnobDirection::CounterClockwise,
            _ => KnobDirection::None,
        }
    }
}
impl Into<u8> for KnobDirection {
    fn into(self) -> u8 {
        match self {
            Self::None => 0b00,
            Self::Clockwise => 0b01,
            Self::CounterClockwise => 0b10,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KnobInputs {
    pub left_switch: SwitchPosition,
    pub left_direction: KnobDirection,
    pub right_switch: SwitchPosition,
    pub right_direction: KnobDirection,
}
impl KnobInputs {
    pub const fn default() -> Self {
        KnobInputs {
            left_switch: SwitchPosition::default(),
            left_direction: KnobDirection::default(),
            right_switch: SwitchPosition::default(),
            right_direction: KnobDirection::default(),
        }
    }

    #[bitmatch]
    pub fn encode(self) -> [u8; 2] {
        let l: u8 = self.left_switch.into();
        let d: u8 = self.left_direction.into();
        let r: u8 = self.right_switch.into();
        let b: u8 = self.right_direction.into();

        [IDENT_KNOB_INPUT, bitpack!("00lddrbb")]
    }

    #[bitmatch]
    pub fn decode(b: &[u8]) -> Result<Self, ()> {
        if b.len() != 2 || *b.get(0).unwrap_or(&b'\0') != IDENT_KNOB_INPUT || b.get(1).is_none() {
            return Err(());
        }

        let w = b.get(1).unwrap();

        #[bitmatch]
        match w {
            "00lddrbb" => Ok(KnobInputs {
                left_switch: l.into(),
                left_direction: match d {
                    0b00 => KnobDirection::None,
                    0b01 => KnobDirection::Clockwise,
                    0b10 => KnobDirection::CounterClockwise,
                    _ => return Err(()),
                },
                right_switch: r.into(),
                right_direction: match b {
                    0b00 => KnobDirection::None,
                    0b01 => KnobDirection::Clockwise,
                    0b10 => KnobDirection::CounterClockwise,
                    _ => return Err(()),
                },
            }),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PedalInputs {
    pub left: SwitchPosition,
    pub middle: SwitchPosition,
    pub right: SwitchPosition,
}
impl PedalInputs {
    pub const fn default() -> Self {
        PedalInputs {
            left: SwitchPosition::default(),
            middle: SwitchPosition::default(),
            right: SwitchPosition::default(),
        }
    }

    #[bitmatch]
    pub fn encode(self) -> [u8; 2] {
        let l: u8 = self.left.into();
        let m: u8 = self.middle.into();
        let r: u8 = self.right.into();

        [IDENT_PEDAL_INPUT, bitpack!("00000lmr")]
    }

    #[bitmatch]
    pub fn decode(b: &[u8]) -> Result<Self, ()> {
        if b.len() != 2 || *b.get(0).unwrap_or(&b'\0') != IDENT_PEDAL_INPUT {
            return Err(());
        }

        let w = b.get(1);
        if w.is_none() {
            return Err(());
        }
        let w = *w.unwrap();

        #[bitmatch]
        match w {
            "00000lmr" => Ok(PedalInputs {
                left: l.into(),
                middle: m.into(),
                right: r.into(),
            }),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
pub enum JBInputs {
    KeyPad(KeyInputs),
    KnobPad(KnobInputs),
    PedalPad(PedalInputs),
}
