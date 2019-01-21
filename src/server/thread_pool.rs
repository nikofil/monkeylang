use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

trait FnBox {
    fn call(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let threads = (0..size).map(|id| Worker::new(id, Arc::clone(&receiver))).collect::<Vec<Worker>>();
        ThreadPool{ threads, sender }
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

struct Worker {
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, job_rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            thread: thread::Builder::new().stack_size(100*1024*1024).spawn(move|| {
                println!("Thread {} started", id);
                loop {
                    let f = job_rx.lock().unwrap().recv().unwrap();
                    println!("Thread {} received job", id);
                    f.call();
                }
            }).unwrap()
        }
    }
}
