use std::{
    sync::{atomic::AtomicU64, Arc},
    time::{Duration, Instant},
};
use threadpool::workgroup::WorkGroup;

fn work_runner(worker_count: usize) -> Duration {
    let site = WorkGroup::new(worker_count);

    let instant_start = Instant::now();
    let collector = Arc::new(AtomicU64::new(0));
    for _ in 0..10000000 {
        let local_collector = collector.clone();
        site.push_single_job(move || {
            let mut val: u128 = 0;
            for i in 0..10000 {
                for j in 0..128 {
                    val *= j;
                }

                val += i;
            }

            local_collector.fetch_add(val as u64, std::sync::atomic::Ordering::Relaxed);
        });
    }

    site.wait_work_consumed();
    site.exit();

    instant_start.elapsed()
}

fn main() {
    for i in vec![1, 2, 4, 8, 16, 32, 64] {
        let duration = work_runner(i);
        println!("Threads[{}]:\t Duration[{:.2?}]", i, duration);
    }
}
