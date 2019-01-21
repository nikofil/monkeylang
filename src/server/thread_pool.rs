use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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

enum Message {
    Job(Job),
    Terminate,
}

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
        self.sender.send(Message::Job(Box::new(f))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        (0..self.threads.len()).for_each(|_| {
            self.sender.send(Message::Terminate).unwrap();
        });
        for thread in &mut self.threads {
            if let Some(thread) = thread.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, job_rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        Worker {
            thread: Some(thread::Builder::new().stack_size(100*1024*1024).spawn(move|| {
                println!("Thread {} started", id);
                loop {
                    let msg = job_rx.lock().unwrap().recv().unwrap();
                    match msg {
                        Message::Job(job) => {
                            println!("Thread {} received job", id);
                            job.call();
                        },
                        Message::Terminate => break,
                    }
                }
                println!("Thread {} exiting", id);
            }).unwrap())
        }
    }
}
