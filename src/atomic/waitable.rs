use std::{marker::PhantomData, ops::Deref, sync::atomic, thread};

use super::{platform::platform, wrapper::AtomicWrapper};

/*------------------------------------------------------------*/

pub struct WaitableAtomic<
    TAtomicType,
    TValueType: Sized + Copy + PartialEq,
    TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>,
> {
    atomic: TAtomicWrapperType,
    _phantom_atomic_type: PhantomData<TAtomicType>,
    _phantom_value_type: PhantomData<TValueType>,
}

impl<
        TAtomicType,
        TValueType: Sized + Copy + PartialEq,
        TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>,
    > WaitableAtomic<TAtomicType, TValueType, TAtomicWrapperType>
{
    pub fn new(initial_value: TValueType) -> Self {
        Self {
            atomic: TAtomicWrapperType::new(initial_value),
            _phantom_atomic_type: PhantomData,
            _phantom_value_type: PhantomData,
        }
    }

    pub fn wake_one(&self) {
        platform::wake_one(&self.atomic);
    }

    pub fn wake_all(&self) {
        platform::wake_all(&self.atomic);
    }

    pub fn wait_exchange(
        &self,
        current: TValueType,
        new: TValueType,
        yield_count: u32,
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

            self.wait_until(move |value| value == current, yield_count);
        }
    }

    pub fn wait_not(&self, expected_not: TValueType, yield_count: u32) -> TValueType {
        let mut loop_count: u32 = 0;
        loop {
            let value = self.atomic.load(atomic::Ordering::Relaxed);
            if value != expected_not {
                break value;
            }

            if loop_count < yield_count {
                thread::yield_now();
                loop_count += 1;
            } else {
                platform::wait_not(&self.atomic, expected_not);
                break self.atomic.load(atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn wait_until<TCheckFuncType>(
        &self,
        check_functor: TCheckFuncType,
        yield_count: u32,
    ) -> TValueType
    where
        TCheckFuncType: Fn(TValueType) -> bool,
    {
        let mut curr = self.atomic.load(atomic::Ordering::Relaxed);
        loop {
            if check_functor(curr) {
                break curr;
            } else {
                curr = self.wait_not(curr, yield_count);
            }
        }
    }
}

impl<
        TAtomicType,
        TValueType: Sized + Copy + PartialEq,
        TAtomicWrapperType: AtomicWrapper<TAtomicType, TValueType>,
    > Deref for WaitableAtomic<TAtomicType, TValueType, TAtomicWrapperType>
{
    type Target = TAtomicType;

    fn deref(&self) -> &Self::Target {
        self.atomic.get_atomic()
    }
}
