use super::job_queue::{JobQueue, QueueEvent};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

pub struct Worker {
    thread_handle: JoinHandle<()>,
}

impl Worker {
    pub fn new(job_queue: Arc<JobQueue>) -> Self {
        let thread_handle = thread::Builder::new()
            .spawn(move || loop {
                match job_queue.wait_event() {
                    QueueEvent::WorkAvailable(job) => {
                        job.run();
                    }
                    QueueEvent::Exit => break,
                }
            })
            .unwrap();

        Self { thread_handle }
    }

    pub fn join(self) {
        self.thread_handle.join().unwrap();
    }
}
