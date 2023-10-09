use std::sync::atomic;

use self::waitable::{AtomicWrapper, WaitableAtomic};

pub mod spinlock;
pub mod waitable;

/*------------------------------------------------------------*/

pub struct AtomicU8 {
    atomic: atomic::AtomicU8,
}

impl AtomicWrapper<atomic::AtomicU8, u8> for AtomicU8 {
    fn new(value: u8) -> Self {
        Self {
            atomic: atomic::AtomicU8::new(value),
        }
    }

    fn get_atomic(&self) -> &atomic::AtomicU8 {
        &self.atomic
    }

    fn load(&self, order: atomic::Ordering) -> u8 {
        self.atomic.load(order)
    }

    fn store(&self, value: u8, order: atomic::Ordering) {
        self.atomic.store(value, order)
    }

    fn compare_exchange(
        &self,
        value_current: u8,
        value_new: u8,
        order_success: atomic::Ordering,
        order_failure: atomic::Ordering,
    ) -> Result<u8, u8> {
        self.atomic
            .compare_exchange(value_current, value_new, order_success, order_failure)
    }
}

pub type WaitableAtomicU8 = WaitableAtomic<atomic::AtomicU8, u8, AtomicU8>;
