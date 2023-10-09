use std::{marker::PhantomData, ops::Deref, sync::atomic, thread};

#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
mod platform;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
#[path = "macos.rs"]
mod platform;

#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(target_os = "freebsd")]
#[path = "freebsd.rs"]
mod platform;

/*------------------------------------------------------------*/

fn wake_one<TAtomicType>(atomic: &TAtomicType) {
    platform::wake_one(atomic);
}

fn wake_all<TAtomicType>(atomic: &TAtomicType) {
    platform::wake_all(atomic);
}

fn wait_not<TAtomicType, TValueType>(atomic: &TAtomicType, expected_not: TValueType) {
    platform::wait_not(atomic, expected_not);
}

/*------------------------------------------------------------*/

pub trait AtomicWrapper<TAtomicType, TValueType: Copy> {
    fn new(value: TValueType) -> Self;
    fn get_atomic(&self) -> &TAtomicType;

    fn load(&self, order: atomic::Ordering) -> TValueType;
    fn store(&self, value: TValueType, order: atomic::Ordering);
    fn compare_exchange(
        &self,
        value_current: TValueType,
        value_new: TValueType,
        order_success: atomic::Ordering,
        order_failure: atomic::Ordering,
    ) -> Result<TValueType, TValueType>;
}

/*------------------------------------------------------------*/

pub struct WaitableAtomic<
    TAtomicType,
    TValueType: Copy,
    TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>,
> {
    atomic: TAtomicWrapperType,
    _phantom_atomic_type: PhantomData<TAtomicType>,
    _phantom_value_type: PhantomData<TValueType>,
}

impl<
        TAtomicType,
        TValueType: Copy + PartialEq,
        TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>,
    > WaitableAtomic<TAtomicType, TValueType, TAtomicWrapperType>
{
    const YIELD_COUNT: u32 = 1000;

    pub fn new(initial_value: TValueType) -> Self {
        Self {
            atomic: TAtomicWrapperType::new(initial_value),
            _phantom_atomic_type: PhantomData,
            _phantom_value_type: PhantomData,
        }
    }

    pub fn wake_one(&self) {
        wake_one(&self.atomic);
    }

    pub fn wake_all(&self) {
        wake_all(&self.atomic);
    }

    pub fn wait_exchange(
        &self,
        current: TValueType,
        new: TValueType,
        order_success: atomic::Ordering,
        order_failure: atomic::Ordering,
    ) {
        loop {
            if let Ok(_) = self
                .atomic
                .compare_exchange(current, new, order_success, order_failure)
            {
                break;
            }

            self.wait_until(move |value| value == current);
        }
    }

    pub fn wait_not(&self, expected_not: TValueType) -> TValueType {
        let mut loop_count: u32 = 0;
        loop {
            let value = self.atomic.load(atomic::Ordering::Relaxed);
            if value != expected_not {
                break value;
            }

            if loop_count < Self::YIELD_COUNT {
                thread::yield_now();
                loop_count += 1;
            } else {
                wait_not(self.atomic.get_atomic(), expected_not);
                break self.atomic.load(atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn wait_until<TCheckFuncType>(&self, check_functor: TCheckFuncType) -> TValueType
    where
        TCheckFuncType: Fn(TValueType) -> bool,
    {
        let mut curr = self.atomic.load(atomic::Ordering::Relaxed);
        loop {
            if check_functor(curr) {
                break curr;
            } else {
                curr = self.wait_not(curr);
            }
        }
    }
}

impl<TAtomicType, TValueType: Copy, TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>>
    Deref for WaitableAtomic<TAtomicType, TValueType, TAtomicWrapperType>
{
    type Target = TAtomicType;

    fn deref(&self) -> &Self::Target {
        self.atomic.get_atomic()
    }
}
