use std::{collections::VecDeque, sync::atomic};

use crate::atomic::{lock::spinlock::SpinLock, WaitableAtomicU8};

use super::Job;

/*------------------------------------------------------------*/

pub enum WorkEvent {
    Available(Box<dyn Job>),
    Exit,
}

/*------------------------------------------------------------*/

pub struct JobQueue {
    job_queue: SpinLock<VecDeque<Box<dyn Job>>>,
    state: WaitableAtomicU8,
}

impl JobQueue {
    const FLAG_WORK_AVAILABLE: u8 = 0b0001;
    const FLAG_EXIT: u8 = 0b0010;

    pub fn new() -> Self {
        Self {
            job_queue: SpinLock::new(VecDeque::new()),
            state: WaitableAtomicU8::new(0),
        }
    }

    pub fn flag_exit(&self) {
        self.state
            .fetch_or(Self::FLAG_EXIT, atomic::Ordering::Release);
        self.state.wake_all();
    }

    pub fn wait_event(&self) -> WorkEvent {
        loop {
            let state = self.state.wait_not(0, 0);
            if state & Self::FLAG_EXIT > 0 {
                break WorkEvent::Exit;
            } else if state & Self::FLAG_WORK_AVAILABLE > 0 {
                let mut guarded_job_queue = self.job_queue.lock();
                match guarded_job_queue.pop_front() {
                    Some(job) => {
                        if guarded_job_queue.is_empty() {
                            self.state
                                .fetch_and(!Self::FLAG_WORK_AVAILABLE, atomic::Ordering::Relaxed);
                        }

                        break WorkEvent::Available(job);
                    }
                    None => {
                        // Last job was claimed by a different worker, return to the waiting state.
                    }
                }
            }
        }
    }

    pub fn push_job<TJob: Job + 'static>(&self, job: TJob) {
        let mut guarded_job_queue = self.job_queue.lock();
        guarded_job_queue.push_back(Box::new(job));
        self.state
            .fetch_or(Self::FLAG_WORK_AVAILABLE, atomic::Ordering::Release);
        self.state.wake_one();
    }
}
