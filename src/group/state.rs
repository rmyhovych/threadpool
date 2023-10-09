use crate::atomic::WaitableAtomicU32;
use std::rc::Rc;

pub struct WorkerGroupState {
    bitset: Rc<WaitableAtomicU32>,
}

impl WorkerGroupState {
    pub fn new() -> Self {
        Self {
            bitset: Rc::new(WaitableAtomicU32::zero()),
        }
    }

    pub fn set_running(&self, worker_index: usize) {
        self.bitset.fetch_or(1 << worker_index);
    }
}
