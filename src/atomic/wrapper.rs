use std::sync::atomic;

/*------------------------------------------------------------*/

pub trait AtomicWrapper<TAtomicType, TValueType: Sized + Copy + PartialEq> {
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
