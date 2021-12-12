mod worker;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use worker::WorkerThread;

pub struct ThreadPool<R> {
    threads: Vec<WorkerThread>,

    work_queue: Arc<Mutex<VecDeque<Box<dyn Fn() -> R + Send + 'static>>>>,
    result_queue: Arc<Mutex<VecDeque<R>>>,
}

impl<R: Sync + Send + 'static> ThreadPool<R> {
    pub fn new(thread_count: usize) -> Self {
        let mut thread_pool = Self {
            threads: Vec::with_capacity(thread_count),

            result_queue: Arc::new(Mutex::new(VecDeque::new())),
            work_queue: Arc::new(Mutex::new(
                VecDeque::<Box<dyn Fn() -> R + Send + 'static>>::new(),
            )),
        };

        for _ in 0..thread_count {
            thread_pool.threads.push(WorkerThread::new(
                &thread_pool.work_queue,
                &thread_pool.result_queue,
            ));
        }

        thread_pool
    }

    pub fn run<F: Fn() -> R + Send + 'static>(&mut self, func: F) {
        let should_wakeup = {
            let mut work_queue_guard = self.work_queue.lock().unwrap();
            let should_wakeup = work_queue_guard.is_empty();
            work_queue_guard.push_back(Box::new(func));

            should_wakeup
        };

        if should_wakeup {
            for t in &self.threads {
                t.wakeup()
            }
        }
    }

    pub fn collect_results(&mut self) -> Vec<R> {
        while !self.work_queue.lock().unwrap().is_empty() {}
        for worker in &self.threads {
            while worker.is_busy() {}
        }

        let mut result_guard = self.result_queue.lock().unwrap();
        (0..result_guard.len())
            .into_iter()
            .map(|_| result_guard.pop_front().unwrap())
            .collect()
    }

    pub fn stop_threads(&mut self) {
        while let Some(thread) = self.threads.pop() {
            thread.join();
        }
    }
}
