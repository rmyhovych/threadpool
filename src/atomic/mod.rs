use std::sync::atomic;

use self::{waitable::WaitableAtomic, wrapper::AtomicWrapper};

pub mod lock;
pub mod waitable;
pub mod wrapper;

mod platform;

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

/*------------------------------------------------------------*/

pub struct AtomicU32 {
    atomic: atomic::AtomicU32,
}

impl AtomicWrapper<atomic::AtomicU32, u32> for AtomicU32 {
    fn new(value: u32) -> Self {
        Self {
            atomic: atomic::AtomicU32::new(value),
        }
    }

    fn get_atomic(&self) -> &atomic::AtomicU32 {
        &self.atomic
    }

    fn load(&self, order: atomic::Ordering) -> u32 {
        self.atomic.load(order)
    }

    fn store(&self, value: u32, order: atomic::Ordering) {
        self.atomic.store(value, order)
    }

    fn compare_exchange(
        &self,
        value_current: u32,
        value_new: u32,
        order_success: atomic::Ordering,
        order_failure: atomic::Ordering,
    ) -> Result<u32, u32> {
        self.atomic
            .compare_exchange(value_current, value_new, order_success, order_failure)
    }
}

pub type WaitableAtomicU32 = WaitableAtomic<atomic::AtomicU32, u32, AtomicU32>;
