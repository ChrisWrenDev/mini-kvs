use crate::{Result, ThreadPoolTrait};
use rayon::ThreadPoolBuilder;

pub struct RayonThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPoolTrait for RayonThreadPool {
    fn new(threads: u32) -> Result<RayonThreadPool> {
        let threads = threads.try_into()?;
        let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;

        Ok(RayonThreadPool { pool })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(job);
    }
}
