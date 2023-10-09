use std::{collections::VecDeque, sync::atomic};

use crate::atomic::{spinlock::SpinLock, WaitableAtomicU8};

use super::job;

/*------------------------------------------------------------*/

pub enum QueueEvent {
    WorkAvailable(Box<dyn job::Job>),
    Exit,
}

/*------------------------------------------------------------*/

enum QueueStateFlag {
    WorkAvailable,
    Exit,
}

struct WaitableQueueState {
    atomic: WaitableAtomicU8,
}

impl WaitableQueueState {
    fn new() -> Self {
        Self {
            atomic: WaitableAtomicU8::new(0),
        }
    }

    fn wait_work_consumed(&self) {
        self.atomic.wait_until(|val| (val & 0b01) == 0);
    }

    fn wait_flag(&self) -> QueueStateFlag {
        let flags = self.atomic.wait_until(|val| (val & 0b11) > 0);
        if flags & 0b10 > 0 {
            QueueStateFlag::Exit
        } else {
            QueueStateFlag::WorkAvailable
        }
    }

    fn notify_work_available(&self) {
        self.atomic.fetch_or(0b01, atomic::Ordering::Acquire);
        self.atomic.wake_one();
    }

    fn clear_work_available(&self) {
        self.atomic.fetch_and(!0b01, atomic::Ordering::Release);
        self.atomic.wake_all();
    }

    fn notify_exit(&self) {
        self.atomic.fetch_or(0b10, atomic::Ordering::Acquire);
        self.atomic.wake_all();
    }
}

/*------------------------------------------------------------*/

pub struct JobQueue {
    job_queue: SpinLock<VecDeque<Box<dyn job::Job>>>,
    queue_state: WaitableQueueState,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            job_queue: SpinLock::new(VecDeque::new()),
            queue_state: WaitableQueueState::new(),
        }
    }

    pub fn flag_exit(&self) {
        self.queue_state.notify_exit();
    }

    pub fn wait_work_consumed(&self) {
        self.queue_state.wait_work_consumed();
    }

    pub fn wait_event(&self) -> QueueEvent {
        loop {
            match self.queue_state.wait_flag() {
                QueueStateFlag::WorkAvailable => {
                    let mut guarded_job_queue = self.job_queue.lock();
                    match guarded_job_queue.pop_front() {
                        Some(job) => {
                            if guarded_job_queue.is_empty() {
                                self.queue_state.clear_work_available();
                            }

                            break QueueEvent::WorkAvailable(job);
                        }
                        None => {}
                    }
                }
                QueueStateFlag::Exit => {
                    break QueueEvent::Exit;
                }
            }
        }
    }

    pub fn push<TJob: job::Job + 'static>(&self, job: TJob) {
        let mut guarded_job_queue = self.job_queue.lock();
        guarded_job_queue.push_back(Box::new(job));
        self.queue_state.notify_work_available();
    }
}
