use std::collections::{HashSet, LinkedList};
use std::sync::RwLock;

use crate::atomic::WaitableAtomicBool;
use crate::resource_id::ResourceID;

pub struct Job {
    id: ResourceID,
    func: Box<dyn Fn()>,

    dependencies: Vec<ResourceID>,
}

impl Job {
    pub fn new<FuncType>(id: ResourceID, func: FuncType) -> Self
    where
        FuncType: Fn() + 'static,
    {
        Self {
            id,
            func: Box::new(func),

            dependencies: Vec::new(),
        }
    }
}

pub struct JobQueue {
    job_queue_state: WaitableAtomicBool,

    queued_jobs: LinkedList<Job>,
    running_jobs: HashSet<ResourceID>,
}

impl JobQueue {
    fn are_dependencies_complete(&self, job: &Job) -> bool {
        RwLock
        false
    }
}
