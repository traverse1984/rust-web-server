use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Pool {
    size: usize,
    tx: Arc<Mutex<Sender<Job>>>,
    rx: Arc<Mutex<Receiver<Job>>>,
}

impl Pool {
    pub fn new(size: usize) -> Pool {
        if size == 0 {
            panic!("There must be at least 1 worker thread");
        }

        let (tx, rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let tx = Arc::new(Mutex::new(tx));
        let rx = Arc::new(Mutex::new(rx));

        let mut pool = Pool { size: 0, tx, rx };
        pool.grow(size);
        pool
    }

    fn worker(&self) {
        let rx = self.rx.clone();
        thread::spawn(move || loop {
            let job = rx.lock().unwrap().recv().unwrap();

            match job.method {
                "shrink" => break,
                _ => (),
            }

            (job.handler)();
        });
    }

    pub fn grow(&mut self, by: usize) -> String {
        let limit = usize::saturating_sub(256, self.size);
        let by = usize::min(by, limit);

        for _ in 0..by {
            self.worker();
        }

        self.size += by;
        format!("Now at {}/256 threads", self.size)
    }

    pub fn shrink(&mut self, by: usize) -> String {
        let limit = self.size - 1;
        let by = usize::min(limit, by);

        for _ in 0..by {
            self.tx.lock().unwrap().send(Job::shrink()).unwrap();
        }

        self.size -= by;
        format!("Now at {}/256 threads", self.size)
    }

    pub fn exec<F>(&self, handler: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.tx.lock().unwrap().send(Job::new(handler)).unwrap();
    }
}

struct Job {
    method: &'static str,
    handler: Box<dyn FnOnce() + Send + Sync + 'static>,
}

impl Job {
    fn new<F>(handler: F) -> Job
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Job {
            method: "exec",
            handler: Box::new(handler),
        }
    }

    fn shrink() -> Job {
        Job {
            method: "shrink",
            handler: Box::new(|| {}),
        }
    }
}
