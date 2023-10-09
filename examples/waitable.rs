use std::{
    sync::{atomic, Arc},
    thread,
    time::Duration,
};
use threadpool::atomic::WaitableAtomicU8;

fn main() {
    let atomic = Arc::new(WaitableAtomicU8::new(0));

    let thread_atomic = atomic.clone();
    let handle = thread::spawn(move || {
        println!("start wait...");
        let value = thread_atomic.wait_not(0);
        println!("end wait {}", value);
    });

    println!("-- 0");
    thread::sleep(Duration::from_millis(100));

    atomic.store(1, atomic::Ordering::SeqCst);
    
    println!("-- 1");
    thread::sleep(Duration::from_millis(100));

    atomic.wake_one();

    println!("-- 2");

    handle.join().unwrap();
}
