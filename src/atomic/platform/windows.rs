use std::mem;
use windows_sys::Win32::System::{
    Threading::{WaitOnAddress, WakeByAddressAll, WakeByAddressSingle},
    WindowsProgramming::INFINITE,
};

use crate::atomic::wrapper::AtomicWrapper;

/*--------------------------------------------------------------------------------*/
#[inline]
pub fn wait_not<
    TAtomicType,
    TValueType: Sized + Copy + PartialEq,
    TWrapper: AtomicWrapper<TAtomicType, TValueType>,
>(
    wrapper: &TWrapper,
    expected_not: TValueType,
) {
    let ptr: *const TAtomicType = wrapper.get_atomic();
    let expected_not_ptr: *const TValueType = &expected_not;
    let address_size = mem::size_of::<TValueType>();
    unsafe { WaitOnAddress(ptr.cast(), expected_not_ptr.cast(), address_size, INFINITE) };
}

#[inline]
pub fn wake_one<
    TAtomicType,
    TValueType: Sized + Copy + PartialEq,
    TWrapper: AtomicWrapper<TAtomicType, TValueType>,
>(
    wrapper: &TWrapper,
) {
    let ptr: *const TAtomicType = wrapper.get_atomic();
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_all<
    TAtomicType,
    TValueType: Sized + Copy + PartialEq,
    TWrapper: AtomicWrapper<TAtomicType, TValueType>,
>(
    wrapper: &TWrapper,
) {
    let ptr: *const TAtomicType = wrapper.get_atomic();
    unsafe { WakeByAddressAll(ptr.cast()) };
}
