use crate::Result;

mod naive;

pub use naive::NaiveThreadPool;

pub trait ThreadPoolTrait {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub enum ThreadPool {
    Naive(NaiveThreadPool),
    //   Shared(SharedThreadPool),
    //   Rayon(RayonThreadPool),
}

pub enum PoolType {
    Naive,
    //  Shared,
    //  Rayon,
}

impl ThreadPool {
    pub fn run(threads: u32) -> Result<Self> {
        let pool_type = PoolType::Naive;

        match pool_type {
            PoolType::Naive => Ok(ThreadPool::Naive(NaiveThreadPool::new(threads)?)),
            //   PoolType::Shared => Ok(ThreadPool::Shared(SharedThreadPool::new(threads)?)),
            //   PoolType::Rayon => Ok(ThreadPool::Rayon(RayonThreadPool::new(threads)?)),
        }
    }
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            ThreadPool::Naive(pool) => pool.spawn(job),
            //   ThreadPool::Shared(pool) => pool.spawn(job),
            //   ThreadPool::Rayon(pool) => pool.spawn(job),
        }
    }
}
