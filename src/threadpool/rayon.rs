use crate::{Result, ThreadPoolTrait};
use rayon::ThreadPoolBuilder;
use std::sync::Arc;

#[derive(Clone)]
pub struct RayonThreadPool {
    pool: Arc<rayon::ThreadPool>,
}

impl ThreadPoolTrait for RayonThreadPool {
    fn new(threads: u32) -> Result<RayonThreadPool> {
        let threads = threads.try_into()?;
        let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;

        Ok(RayonThreadPool {
            pool: Arc::new(pool),
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(job);
    }
}
