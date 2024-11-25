// All the utilities for the communication protocol

pub const CMD_GREET: u8 = b'\x05';
pub const CMD_GET_INPUT_KEYS: u8 = b'\x30';
pub const CMD_UPDATE: u8 = b'\x38';
pub const CMD_DISCONNECT: u8 = b'\x39';
pub const CMD_NEGATIVE_ACK: u8 = b'\x15';
pub const CMD_UNKNOWN: u8 = b'?';

pub const CMD_DEVICE: u8 = b'U';
pub const CMD_END: &[u8] = b"\r\n";

pub const RSP_LINK_HEADER: u8 = b'L';
pub const RSP_LINK_DELIMITER: u8 = b',';

pub const RSP_INPUT_HEADER: u8 = b'I';

pub const RSP_UNKNOWN: u8 = b'?';
pub const RSP_DISCONNECTED: u8 = b'\x04';

pub const RSP_END: &[u8] = b"\r\n\r\n";

#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Command {
    Greeting,
    GetInputKeys,
    Update,
    Disconnect,
    NegativeAck,
    Unknown,
}
impl Command {
    pub fn decode(w: u8) -> Self {
        if w == CMD_GREET {
            Self::Greeting
        } else if w == CMD_GET_INPUT_KEYS {
            Self::GetInputKeys
        } else if w == CMD_UPDATE {
            Self::Update
        } else if w == CMD_DISCONNECT {
            Self::Update
        } else if w == CMD_NEGATIVE_ACK {
            Self::NegativeAck
        } else {
            Self::Unknown
        }
    }
}
