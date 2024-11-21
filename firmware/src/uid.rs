//! Handles flash Unique ID

use core::ptr::addr_of;

use rp2040_flash::flash;

static mut UID_BYTES: [u8; 16] = [0u8; 16];

fn num_to_hex(x: u8) -> u8 {
    match x {
        0x0 => b'0',
        0x1 => b'1',
        0x2 => b'2',
        0x3 => b'3',
        0x4 => b'4',
        0x5 => b'5',
        0x6 => b'6',
        0x7 => b'7',
        0x8 => b'8',
        0x9 => b'9',
        0xA => b'A',
        0xB => b'B',
        0xC => b'C',
        0xD => b'D',
        0xE => b'E',
        0xF => b'F',
        _ => b'?',
    }
}

fn get_flash_uid_bytes() -> [u8; 8] {
    let mut unique_id = [0u8; 8];
    unsafe { cortex_m::interrupt::free(|_cs| flash::flash_unique_id(&mut unique_id, true)) };
    unique_id
}

pub fn get_flash_uid() -> &'static str {
    if unsafe { UID_BYTES[0] == 0 } {
        get_flash_uid_bytes()
            .iter()
            .flat_map(|n| [n / 16, n % 16])
            .enumerate()
            .for_each(|(i, n)| unsafe {
                UID_BYTES[i] = num_to_hex(n);
            });
        // Due to how this conversion works, we can guarantee that the string is
        // valid UTF-8 and that no undefined-behavior happens. Despite this, the
        // compiler (correctly) tries to warn and prevent this.
    }

    // The line below is considered bad practice.
    unsafe { core::str::from_utf8(&*addr_of!(UID_BYTES)).unwrap() }
}
