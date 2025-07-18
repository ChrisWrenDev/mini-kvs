use crate::Result;

mod naive;
mod queue;

pub use naive::NaiveThreadPool;
pub use queue::QueueThreadPool;

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
    Queue(QueueThreadPool),
    //   Rayon(RayonThreadPool),
}

pub enum PoolType {
    Naive,
    Queue,
    //  Rayon,
}

impl ThreadPool {
    pub fn run(threads: u32) -> Result<Self> {
        let pool_type = PoolType::Naive;

        match pool_type {
            PoolType::Naive => Ok(ThreadPool::Naive(NaiveThreadPool::new(threads)?)),
            PoolType::Queue => Ok(ThreadPool::Queue(QueueThreadPool::new(threads)?)),
            //   PoolType::Rayon => Ok(ThreadPool::Rayon(RayonThreadPool::new(threads)?)),
        }
    }
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            ThreadPool::Naive(pool) => pool.spawn(job),
            ThreadPool::Queue(pool) => pool.spawn(job),
            //   ThreadPool::Rayon(pool) => pool.spawn(job),
        }
    }
}
