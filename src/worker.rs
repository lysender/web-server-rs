use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use tracing::info;

pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            // Just keep looping and waiting for a job to execute
            loop {
                let job = receiver
                    .lock()
                    .unwrap()
                    .recv()
                    .expect("Could not receive job");

                info!("Worker {} got a job; executing.", id);
                job();
            }
        });

        Worker { id, thread }
    }
}

impl WorkerPool {
    pub fn new(size: usize) -> WorkerPool {
        assert!(size > 0 && size < 1000);

        let (sender, receiver) = mpsc::channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&shared_receiver)));
        }

        WorkerPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
