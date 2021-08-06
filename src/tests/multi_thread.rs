use super::*;

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::io;
use std::io::Write;

pub mod print_output {
    use super::*;
    use super::super::*;
    use crate::*;

    // Option, Ref sequence, Qry sequence, alignment function
    pub type Job = (AlignmentOption, Vec<u8>, Vec<u8>, Box<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static>);

    pub struct Executor {
        pub job_sender: Option<mpsc::Sender<Job>>,
        workers: Vec<Worker>,
    }
    impl Executor {
        pub fn new(pool_size: usize) -> Self {
            let (job_sender, job_receiver) = mpsc::channel();
            let job_receiver = Arc::new(Mutex::new(job_receiver));

            let mut workers = Vec::with_capacity(pool_size);

            for id in 0..pool_size {
                workers.push(
                    Worker::new(id, job_receiver.clone())
                );
            }
            Self {
                job_sender: Some(job_sender),
                workers,
            }
        }
        pub fn get_sender(&mut self) -> Option<mpsc::Sender<Job>> {
            self.job_sender.take()
        }
    }
    impl Drop for Executor {
        fn drop(&mut self) {
            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
    
    struct Worker {
        id: usize,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
        fn new(id: usize, job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
            let thread = thread::spawn(move || loop {
                let (alignment_option, ref_seq, qry_seq, f) = match job_receiver.lock().expect("Job receiving err").recv() {
                    Ok(job) => {
                        job
                    }
                    Err(_) => {
                        println!("Shutting down thread {}", id);
                        break;
                    }
                };
                f(alignment_option, ref_seq, qry_seq);
            });
            Self {
                id: id,
                thread: Some(thread),
            }
        }
    }
}