use parking_lot::Mutex;
use std::io::{BufWriter, Write};
use std::sync::{mpsc, Arc};
use std::thread;

use sigalign::algorithms::Algorithm;
use sigalign::{Aligner, Reference};
use sigalign_utils::sequence_manipulation::reverse_complementary::reverse_complement_of_dna_sequence_in_place;

use crate::alignment::write_results::ResFormatter;

use super::QueryReader;

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_senders: Vec<mpsc::SyncSender<Job>>,
    res_receiver: mpsc::Receiver<JobCompleteSign>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

struct Job {
    reference: Arc<Reference>,
    query_reader: Arc<Mutex<QueryReader>>,
}
struct JobCompleteSign;

impl ThreadPool {
    pub fn new<A: Algorithm + 'static>(
        num_workers: usize,
        batch_size: usize,
        aligner: Aligner<A>,
        with_reverse_complementary: bool,
        output_is_sam: bool,
    ) -> Self {
        let mut workers = Vec::with_capacity(num_workers);

        let mut job_senders = Vec::with_capacity(num_workers);
        let (res_sender, res_receiver) = mpsc::channel();

        // Spawn threads
        for idx in 0..num_workers {
            let (job_sender, job_receiver) = mpsc::sync_channel(1);
            job_senders.push(job_sender);
            let job_receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::new(Mutex::new(job_receiver));

            workers.push(Worker::new(
                idx + 1,
                batch_size,
                aligner.clone(),
                job_receiver.clone(),
                res_sender.clone(),
                with_reverse_complementary,
                output_is_sam,
            ));
        }

        Self {
            workers,
            job_senders,
            res_receiver,
        }
    }
    pub fn execute(&self, reference: Reference, query_reader: QueryReader) {
        // Send jobs
        let reference = Arc::new(reference);
        let query_reader = Arc::new(Mutex::new(query_reader));
        for job_sender in &self.job_senders {
            job_sender
                .send(Job {
                    reference: reference.clone(),
                    query_reader: query_reader.clone(),
                })
                .unwrap();
        }

        // Wait for all jobs to complete
        for _ in 0..self.job_senders.len() {
            self.res_receiver.recv().unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.job_senders.clear();

        for worker in self.workers.drain(..) {
            drop(worker);
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
        with_reverse_complementary: bool,
        output_is_sam: bool,
    ) -> Worker {
        eprintln!("[Worker {}] spawned", id);

        // Vec of (sequence, label) buffer
        let mut query_buffers: Vec<(Vec<u8>, String)> =
            vec![(Vec::new(), String::new()); batch_size];
        // Bytes to write as results
        let results_buffer = Vec::new();
        let mut buf_writer = BufWriter::new(results_buffer);

        let thread = thread::spawn(move || {
            let mut res_formatter = ResFormatter::new(
                output_is_sam,
            );
            let stdout = std::io::stdout();

            loop {
                let Job {
                    reference,
                    query_reader,
                } = match job_receiver.lock().recv() {
                    Ok(job) => job,
                    Err(_) => {
                        break;
                    }
                };

                loop {
                    // Fill buffers
                    let optional_last_buffer_index = {
                        let mut last_buffer_index = None;
                        let mut query_reader = query_reader.lock();
                        for (buffer_index, (sequence_buffer, label_buffer)) in
                            query_buffers.iter_mut().enumerate()
                        {
                            if let Err(_) =
                                query_reader.fill_record_buffer(sequence_buffer, label_buffer)
                            {
                                last_buffer_index = Some(buffer_index);
                                break;
                            }
                        }
                        last_buffer_index
                    };

                    // Align
                    let last_buffer_index =
                        optional_last_buffer_index.unwrap_or(query_buffers.len());
                    // Forward
                    for (query, label) in query_buffers[..last_buffer_index].iter() {
                        let result = aligner.align(query, &reference);
                        let labeled_result = reference.label_query_alignment(result);
                        res_formatter.write_record(
                            &mut buf_writer,
                            &labeled_result,
                            label,
                            query.len() as u32,
                            true,
                        ).unwrap();
                    }
                    // Reverse
                    if with_reverse_complementary {
                        query_buffers[..last_buffer_index]
                            .iter_mut()
                            .for_each(|(query, label)| {
                                reverse_complement_of_dna_sequence_in_place(query);

                                let result = aligner.align(query, &reference);
                                let labeled_result = reference.label_query_alignment(result);
                                res_formatter.write_record(&mut buf_writer,
                                    &labeled_result,
                                    label,
                                    query.len() as u32,
                                    false,
                                ).unwrap();
                            });
                    }

                    // Write results
                    {
                        buf_writer.flush().unwrap();
                        let inner = buf_writer.get_mut();

                        let mut lock = stdout.lock();
                        lock.write_all(&inner).unwrap();
                        inner.clear();
                    }

                    // If no more records: break
                    if optional_last_buffer_index.is_some() {
                        break;
                    }
                }

                res_sender.send(JobCompleteSign).unwrap();
                eprintln!("[Worker {}] job completed", id);
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
        eprintln!("[Worker {}] dropped", self.id);
    }
}
