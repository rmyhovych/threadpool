use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic,
};

use crate::atomic::WaitableAtomicU32;

/*------------------------------------------------------------*/

pub struct ReadLockGuard<'a, TObject> {
    object: &'a TObject,
    control: &'a WaitableAtomicU32,
}

impl<'a, TObject> Deref for ReadLockGuard<'a, TObject> {
    type Target = TObject;

    fn deref(&self) -> &Self::Target {
        self.object
    }
}

impl<'a, TObject> Drop for ReadLockGuard<'a, TObject> {
    fn drop(&mut self) {
        self.control.fetch_sub(1, atomic::Ordering::Release);
        self.control.wake_one();
    }
}

/*------------------------------------------------------------*/

pub struct WriteLockGuard<'a, TObject> {
    object: &'a mut TObject,
    control: &'a WaitableAtomicU32,
}

impl<'a, TObject> Deref for WriteLockGuard<'a, TObject> {
    type Target = TObject;

    fn deref(&self) -> &Self::Target {
        self.object
    }
}

impl<'a, TObject> DerefMut for WriteLockGuard<'a, TObject> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object
    }
}

impl<'a, TObject> Drop for WriteLockGuard<'a, TObject> {
    fn drop(&mut self) {
        self.control.store(0, atomic::Ordering::Release);
        self.control.wake_all();
    }
}

/*------------------------------------------------------------*/

pub struct SpinRWLock<TObject> {
    object: UnsafeCell<TObject>,
    control: WaitableAtomicU32,
}

impl<TObject> SpinRWLock<TObject> {
    const MASK_WRITE: u32 = 1 << 31;

    pub fn new(object: TObject) -> Self {
        Self {
            object: UnsafeCell::new(object),
            control: WaitableAtomicU32::new(0),
        }
    }

    pub fn lock_read<'a>(&'a self) -> ReadLockGuard<'a, TObject> {
        loop {
            let value = self
                .control
                .wait_until(|val| (val & Self::MASK_WRITE) == 0, 1000);
            match self.control.compare_exchange(
                value,
                value + 1,
                atomic::Ordering::Acquire,
                atomic::Ordering::Relaxed,
            ) {
                Ok(_) => {
                    break unsafe {
                        let object_ptr = self.object.get();
                        let object = &*object_ptr;

                        ReadLockGuard {
                            object,
                            control: &self.control,
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    pub fn lock_write<'a>(&'a self) -> WriteLockGuard<'a, TObject> {
        self.control.wait_exchange(
            0,
            Self::MASK_WRITE,
            1000,
            atomic::Ordering::Acquire,
            atomic::Ordering::Relaxed,
        );

        unsafe {
            let object_ptr = self.object.get();
            let object = &mut *(object_ptr);

            WriteLockGuard {
                object,
                control: &self.control,
            }
        }
    }
}
