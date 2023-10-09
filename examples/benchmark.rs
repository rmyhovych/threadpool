use std::{
    sync::{atomic, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use threadpool::atomic::{spinlock::SpinLock, WaitableAtomicU8};

const THREAD_COUNT: u32 = 8;

fn main() {
    let mut duration_min = Duration::MAX;
    let mut duration_max = Duration::ZERO;
    let mut duration_average = Duration::ZERO;
    for _ in 0..10 {
        let value = Arc::new(Mutex::new(0));

        let waiter = Arc::new(WaitableAtomicU8::new(0));
        let mut thread_handles = Vec::<thread::JoinHandle<()>>::new();
        for _ in 0..THREAD_COUNT {
            let thread_value = value.clone();
            let thread_waiter = waiter.clone();
            let handle = thread::spawn(move || {
                thread_waiter.wait_not(0);
                for _ in 0..10000 {
                    {
                        let mut guarded_value = thread_value.lock();
                        let val = guarded_value.as_deref_mut().unwrap();
                        for _ in 0..100 {
                            *val += 1;
                        }
                    }
                }
            });

            thread_handles.push(handle);
        }

        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        waiter.store(1, atomic::Ordering::Release);
        waiter.wake_all();

        for handle in thread_handles {
            handle.join().unwrap();
        }

        let elapsed = now.elapsed();
        duration_average += elapsed;
        duration_min = duration_min.min(elapsed);
        duration_max = duration_max.max(elapsed);
    }

    println!(
        "Min[{:.2?}] Max[{:.2?}] Average[{:.2?}]",
        duration_min,
        duration_max,
        duration_average.div_f64(10.0)
    );
}
