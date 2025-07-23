use crate::{Result, ThreadPoolTrait};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tracing::error;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct QueueThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPoolTrait for QueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        assert!(threads > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let threads = threads.try_into()?;
        let mut workers = Vec::with_capacity(threads);

        for id in 0..threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(QueueThreadPool {
            workers,
            sender: Some(sender),
        })
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);

        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(job) {
                error!("Failed to send job to worker: {err}");
            }
        }
    }
}

impl Drop for QueueThreadPool {
    fn drop(&mut self) {
        // Drop the sender to signal workers to shut down
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(err) = thread.join() {
                    error!("Worker {} panicked: {:?}", worker.id, err);
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        // println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        // println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
