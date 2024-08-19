//! Utility functions

pub fn nop_loop(n: u8) {
    for _n in 0..n {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}
