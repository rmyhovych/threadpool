use std::mem;
use windows_sys::Win32::System::{
    Threading::{WaitOnAddress, WakeByAddressAll, WakeByAddressSingle},
    WindowsProgramming::INFINITE,
};

/*--------------------------------------------------------------------------------*/

#[inline]
pub fn wait_not<TAtomicType, TValueType: Sized>(a: &TAtomicType, expected_not: TValueType) {
    let ptr = a as *const TAtomicType;
    let expected_not_ptr: *const TValueType = &expected_not;
    let address_size = mem::size_of::<TValueType>();
    unsafe { WaitOnAddress(ptr.cast(), expected_not_ptr.cast(), address_size, INFINITE) };
}

#[inline]
pub fn wake_one<TAtomicType>(a: &TAtomicType) {
    let ptr = a as *const TAtomicType;
    unsafe { WakeByAddressSingle(ptr.cast()) };
}

#[inline]
pub fn wake_all<TAtomicType>(a: &TAtomicType) {
    let ptr = a as *const TAtomicType;
    unsafe { WakeByAddressAll(ptr.cast()) };
}
