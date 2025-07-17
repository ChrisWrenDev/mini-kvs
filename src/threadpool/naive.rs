use crate::{Result, ThreadPoolTrait};
use std::thread;

pub struct NaiveThreadPool;

impl ThreadPoolTrait for NaiveThreadPool {
    fn new(threads: u32) -> Result<NaiveThreadPool> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
