use std::{
    ops::{Deref, DerefMut},
    sync::atomic, cell::UnsafeCell,
};

use super::WaitableAtomicU8;

/*------------------------------------------------------------*/

pub struct SpinLockGuard<'a, TObject> {
    atomic: &'a WaitableAtomicU8,
    object: &'a mut TObject,
}

impl<'a, TObject> Deref for SpinLockGuard<'a, TObject> {
    type Target = TObject;

    fn deref(&self) -> &Self::Target {
        self.object
    }
}

impl<'a, TObject> DerefMut for SpinLockGuard<'a, TObject> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object
    }
}

impl<'a, TObject> Drop for SpinLockGuard<'a, TObject> {
    fn drop(&mut self) {
        self.atomic.store(0, atomic::Ordering::Release);
        self.atomic.wake_one();
    }
}

/*------------------------------------------------------------*/

pub struct SpinLock<TObject> {
    object: UnsafeCell<TObject>,
    atomic: WaitableAtomicU8,
}

impl<TObject> SpinLock<TObject> {
    pub fn new(object: TObject) -> Self {
        Self {
            atomic: WaitableAtomicU8::new(0),
            object: UnsafeCell::new(object),
        }
    }

    pub fn lock<'a>(&'a self) -> SpinLockGuard<'a, TObject> {
        self.atomic.wait_exchange(
            0,
            1,
            1000,
            atomic::Ordering::Acquire,
            atomic::Ordering::Relaxed,
        );
        unsafe {
            let object_ptr = self.object.get();
            let object = &mut *(object_ptr);

            SpinLockGuard {
                atomic: &self.atomic,
                object,
            }
        }
    }
}

unsafe impl<TObject> Sync for SpinLock<TObject> {}
