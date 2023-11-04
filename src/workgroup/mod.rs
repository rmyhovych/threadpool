mod job_queue;
mod worker;

use std::sync::Arc;

use job_queue::JobQueue;
use worker::Worker;

/*------------------------------------------*/

pub trait Job: Send {
    fn run(&self);
}

/*------------------------------------------*/

struct SingleJob<TFuncType: FnOnce()> {
    func: TFuncType,
}

unsafe impl<TFuncType: FnOnce()> Send for SingleJob<TFuncType> {}

impl<TFuncType: FnOnce() + 'static> SingleJob<TFuncType> {
    pub fn new(func: TFuncType) -> Self {
        Self { func }
    }
}

impl<TFuncType: FnOnce()> Job for SingleJob<TFuncType> {
    fn run(&self) {
        (self.func)();
    }
}

/*------------------------------------------*/

struct ProducingJob<TResultType, TFuncType: FnOnce() -> TResultType + 'static> {
    func: TFuncType,
}

unsafe impl<TResultType, TFuncType: FnOnce() -> TResultType + 'static> Send
    for ProducingJob<TResultType, TFuncType>
{
}

impl<TResultType, TFuncType: FnOnce() -> TResultType + 'static>
    ProducingJob<TResultType, TFuncType>
{
    pub fn new(func: TFuncType) -> Self {
        Self { func }
    }
}

impl<TResultType, TFuncType: FnOnce() -> TResultType + 'static> Job
    for ProducingJob<TResultType, TFuncType>
{
    fn run(&self) {
        (self.func)();
    }
}

/*------------------------------------------*/

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
        self.job_queue.push_job(job);
    }

    pub fn push_single_job<TJob: FnOnce() + 'static>(&self, job: TJob) {
        self.push_job(SingleJob::new(job));
    }

    pub fn exit(self) {
        self.job_queue.flag_exit();
        for worker in self.workers {
            worker.join();
        }
    }
}
