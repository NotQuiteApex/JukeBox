use core::cell::UnsafeCell;

use rp_pico::hal::sio::{Spinlock, SpinlockValid};

pub struct Mutex<const N: usize, T: ?Sized>
where
    Spinlock<N>: SpinlockValid,
{
    data: UnsafeCell<T>,
}

unsafe impl<const N: usize, T: ?Sized> Send for Mutex<N, T> where Spinlock<N>: SpinlockValid {}
unsafe impl<const N: usize, T: ?Sized> Sync for Mutex<N, T> where Spinlock<N>: SpinlockValid {}

impl<const N: usize, T> Mutex<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    pub const fn new(data: T) -> Self {
        Mutex {
            data: UnsafeCell::new(data),
        }
    }

    pub fn with_lock(&self, f: impl FnOnce(&T) -> ()) {
        let _lock = Spinlock::<N>::claim();
        cortex_m::asm::dmb();
        f(unsafe { &*self.data.get() });
        cortex_m::asm::dmb();
    }

    pub fn with_mut_lock(&self, f: impl FnOnce(&mut T) -> ()) {
        let _lock = Spinlock::<N>::claim();
        cortex_m::asm::dmb();
        f(unsafe { &mut *self.data.get() });
        cortex_m::asm::dmb();
    }
}
