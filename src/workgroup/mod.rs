pub mod job;
mod job_queue;
mod worker;

use std::sync::Arc;

use self::{job::SingleJob, job_queue::JobQueue, worker::Worker};

pub struct WorkGroup {
    job_queue: Arc<JobQueue>,
    workers: Vec<Worker>,
}

impl WorkGroup {
    pub fn new(worker_count: usize) -> Self {
        debug_assert!(worker_count > 0, "Worker count can't be 0.");

        let job_queue = Arc::new(JobQueue::new());

        let workers = (0..worker_count.max(1))
            .map(|_| Worker::new(job_queue.clone()))
            .collect();

        Self { job_queue, workers }
    }

    pub fn push_job<TJob: job::Job + 'static>(&self, job: TJob) {
        self.job_queue.push(job);
    }

    pub fn push_single_job<TJob: Fn() + 'static>(&self, job: TJob) {
        self.job_queue.push(SingleJob::new(job));
    }

    pub fn wait_work_consumed(&self) {
        self.job_queue.wait_work_consumed();
    }

    pub fn exit(self) {
        self.job_queue.flag_exit();
        for worker in self.workers {
            worker.join();
        }
    }
}
