use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct WorkerThread {
    handle: JoinHandle<()>,

    should_kill: Arc<AtomicBool>,
    should_sleep: Arc<AtomicBool>,

    working_flag: Arc<AtomicBool>,
}

impl WorkerThread {
    pub fn new<R: Send + 'static>(
        work_queue: &Arc<Mutex<VecDeque<Box<dyn Fn() -> R + Send + 'static>>>>,
        result_queue: &Arc<Mutex<VecDeque<R>>>,
    ) -> Self {
        let work_queue_local = Arc::clone(work_queue);
        let result_queue_local = Arc::clone(result_queue);

        let should_kill = Arc::new(AtomicBool::new(false));
        let should_kill_local = Arc::clone(&should_kill);

        let should_sleep = Arc::new(AtomicBool::new(false));
        let should_sleep_local = Arc::clone(&should_sleep);

        let working_flag = Arc::new(AtomicBool::new(false));
        let working_flag_local = Arc::clone(&working_flag);

        let handle = thread::spawn(move || {
            while !should_kill_local.load(Ordering::Relaxed) {
                let work: Option<Box<dyn Fn() -> R + Send + 'static>> = {
                    let mut work_queue_guard = work_queue_local.lock().unwrap();
                    if !work_queue_guard.is_empty() {
                        work_queue_guard.pop_front()
                    } else {
                        should_sleep_local.store(true, Ordering::Relaxed);
                        None
                    }
                };

                match work {
                    Some(work_fn) => {
                        working_flag_local.store(true, Ordering::Relaxed);
                        let result = work_fn.as_ref()();
                        result_queue_local.lock().unwrap().push_back(result);
                        working_flag_local.store(false, Ordering::Relaxed);
                    }
                    None => {
                        while should_sleep_local.load(Ordering::Relaxed) {
                            thread::park();
                        }
                    }
                }
            }
        });

        Self {
            handle,

            should_kill: Arc::clone(&should_kill),
            should_sleep: Arc::clone(&should_sleep),

            working_flag: Arc::clone(&working_flag),
        }
    }

    pub fn is_busy(&self) -> bool {
        return self.working_flag.load(Ordering::Relaxed);
    }

    pub fn wakeup(&self) {
        self.should_sleep.store(false, Ordering::Relaxed);
        self.handle.thread().unpark();
    }

    pub fn join(self) {
        self.should_kill.store(true, Ordering::Relaxed);
        self.wakeup();
        self.handle.join().unwrap();
    }
}
