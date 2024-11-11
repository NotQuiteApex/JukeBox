use defmt::info;
use rp_pico::hal::sio::{Spinlock, Spinlock0, SpinlockValid};

#[derive(Clone, Copy)]
pub struct Mutex<const N: usize, T>
where
    Spinlock<N>: SpinlockValid,
{
    data: *mut T,
}

unsafe impl<const N: usize, T> Send for Mutex<N, T> where Spinlock<N>: SpinlockValid {}

impl<const N: usize, T> Mutex<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    pub fn new(data: *mut T) -> Self {
        Mutex { data: data }
    }

    pub fn with_lock(&self, f: impl FnOnce(&T) -> ()) {
        let _lock = Spinlock::<N>::claim();
        cortex_m::asm::dmb();
        f(unsafe { &*self.data });
        cortex_m::asm::dmb();
    }

    pub fn with_mut_lock(&mut self, f: impl FnOnce(&mut T) -> ()) {
        let _lock = Spinlock::<N>::claim();
        cortex_m::asm::dmb();
        f(unsafe { &mut *self.data });
        cortex_m::asm::dmb();
    }
}
