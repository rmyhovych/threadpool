use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic,
};

use crate::atomic::WaitableAtomicU8;

/*------------------------------------------------------------*/

pub struct SpinLockGuard<'a, TObject> {
    object: &'a mut TObject,
    control: &'a WaitableAtomicU8,
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
        self.control.store(0, atomic::Ordering::Release);
        self.control.wake_one();
    }
}

/*------------------------------------------------------------*/

pub struct SpinLock<TObject> {
    object: UnsafeCell<TObject>,
    control: WaitableAtomicU8,
}

impl<TObject> SpinLock<TObject> {
    pub fn new(object: TObject) -> Self {
        Self {
            control: WaitableAtomicU8::new(0),
            object: UnsafeCell::new(object),
        }
    }

    pub fn lock<'a>(&'a self) -> SpinLockGuard<'a, TObject> {
        self.control.wait_exchange(
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
                object,
                control: &self.control,
            }
        }
    }
}

unsafe impl<TObject> Sync for SpinLock<TObject> {}
