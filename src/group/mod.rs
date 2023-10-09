mod job;
mod state;
mod worker;

use std::collections::LinkedList;

use crate::atomic::WaitableAtomicBool;
use crate::resource_id::ResourceID;

use job::Job;
use worker::Worker;

pub struct ThreadGroup {
    worker_state: state::WorkerGroupState,
    workers: Vec<Worker>,


    running_job_ids: LinkedList<ResourceID>,
}
