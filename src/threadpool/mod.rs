use crate::Result;
use clap::ValueEnum;
use std::fmt::{self, Display, Formatter};
use tracing::info;

mod naive;
mod queue;
mod rayon;

pub use naive::NaiveThreadPool;
pub use queue::QueueThreadPool;
pub use rayon::RayonThreadPool;

pub trait ThreadPoolTrait: Clone + Send + Sync + 'static {
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
    Rayon(RayonThreadPool),
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PoolType {
    Naive,
    Queue,
    Rayon,
}

impl Display for PoolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            PoolType::Naive => "naive",
            PoolType::Queue => "queue",
            PoolType::Rayon => "rayon",
        };
        write!(f, "{}", s)
    }
}

impl ThreadPool {
    pub fn run(pool: PoolType, threads: u32) -> Result<Self> {
        info!("Thread Pool Type: {}", pool.to_string());

        match pool {
            PoolType::Naive => Ok(ThreadPool::Naive(NaiveThreadPool::new(threads)?)),
            PoolType::Queue => Ok(ThreadPool::Queue(QueueThreadPool::new(threads)?)),
            PoolType::Rayon => Ok(ThreadPool::Rayon(RayonThreadPool::new(threads)?)),
        }
    }
    pub fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            ThreadPool::Naive(pool) => pool.spawn(job),
            ThreadPool::Queue(pool) => pool.spawn(job),
            ThreadPool::Rayon(pool) => pool.spawn(job),
        }
    }
}
