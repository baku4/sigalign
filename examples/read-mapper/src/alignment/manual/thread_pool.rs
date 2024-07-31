use std::io::Write as _;
use std::path::PathBuf;
use std::thread;
use std::sync::{mpsc, Arc};
use parking_lot::Mutex;

use sigalign::algorithms::Algorithm;
use sigalign::results::QueryAlignment;
use sigalign::{Aligner, Reference};

use crate::alignment::query_reader::{self, QueryReader};

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_senders: Vec<mpsc::SyncSender<Job>>,
    res_receiver: mpsc::Receiver<JobCompleteSign>,
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

struct Job {
    reference: Arc<Reference>,
    query_reader: Arc<Mutex<QueryReader>>,
    reference_index: u32,
}
struct JobCompleteSign;


impl ThreadPool {
    pub fn new<A: Algorithm + 'static>(
        num_workers: usize,
        batch_size: usize,
        aligner: Aligner<A>,
    ) -> Self {
        let mut workers = Vec::with_capacity(num_workers);

        let mut job_senders = Vec::with_capacity(num_workers);
        let (res_sender, res_receiver) = mpsc::channel();
        
        // Spawn threads
        for id in 0..num_workers {
            let (job_sender, job_receiver) = mpsc::sync_channel(1);
            job_senders.push(job_sender);
            let job_receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::new(Mutex::new(job_receiver));

            workers.push(Worker::new(
                id,
                batch_size,
                aligner.clone(),
                job_receiver.clone(),
                res_sender.clone(),
            ));
        }

        Self {
            workers,
            job_senders,
            res_receiver,
        }
    }
    pub fn execute(
        &self,
        reference: Reference,
        reference_index: u32,
        query_reader: QueryReader,
    ) {
        // Send jobs
        let reference = Arc::new(reference);
        let query_reader = Arc::new(Mutex::new(query_reader));
        for job_sender in &self.job_senders {
            job_sender.send(Job {
                reference: reference.clone(),
                query_reader: query_reader.clone(),
                reference_index,
            }).unwrap();
        }

        // Wait for all jobs to complete
        for _ in 0..self.job_senders.len() {
            self.res_receiver.recv().unwrap();
        }
    }
}

impl Worker {
    fn new<A: Algorithm + 'static>(
        id: usize,
        batch_size: usize,
        mut aligner: Aligner<A>,
        job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        res_sender: mpsc::Sender<JobCompleteSign>,
    ) -> Worker {
        eprintln!("[Worker {}] spawned", id);

        // Vec of (Sequence, label) buffer
        let mut buffers: Vec<(Vec<u8>, String)> = vec![(Vec::new(), String::new()); batch_size];
        let mut results = Vec::with_capacity(batch_size);

        let thread = thread::spawn(move || {
            let stdout = std::io::stdout();

            loop {
                let Job {
                    reference,
                    query_reader,
                    reference_index,
                } = match job_receiver.lock().recv() {
                    Ok(job) => job,
                    Err(_) => {
                        break;
                    },
                };

                loop {
                    // Fill buffers
                    let last_buffer_index = {
                        let mut last_buffer_index = None;
                        let mut query_reader = query_reader.lock();
                        for (
                            buffer_index,
                            (sequence_buffer, label_buffer)
                        ) in buffers.iter_mut().enumerate() {
                            if let Err(_) = query_reader.fill_record_buffer(sequence_buffer, label_buffer) {
                                last_buffer_index = Some(buffer_index);
                                break;
                            }
                        }
                        last_buffer_index
                    };

                    // Align
                    for (query, label) in buffers[
                        ..last_buffer_index.unwrap_or_else(|| buffers.len())
                    ].iter() {
                        let result = aligner.align(query, &reference);
                        let labeled_result = reference.label_query_alignment(result);
                        results.push((label.clone(), labeled_result));
                    }

                    res_sender.send(JobCompleteSign).unwrap();
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}
