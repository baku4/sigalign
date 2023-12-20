use std::io::Write as _;
use std::path::PathBuf;
use std::thread;
use std::sync::{mpsc, Mutex, Arc};

use crate::Result;
use sigalign::{
    Aligner, Reference,
    results::{TargetAlignmentResult, AlignmentOperations, AlignmentOperation},
};
use sigalign_utils::{
    sequence_reader::{
        SeqRecord, IdRecord,
        fasta::FastaReader,
    },
    sequence_manipulation::reverse_complementary::reverse_complement_of_dna_sequence_in_place,
};
use sigalign_impl::sequence_storage::in_memory::InMemoryBuffer;

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_sender: Option<mpsc::SyncSender<Job>>,
    res_receiver: mpsc::Receiver<JobCompleteSign>,
    input_fasta_file: PathBuf,
    batch_size: usize,
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

pub struct Job {
    reference: Arc<Reference>,
    reference_index: u32,
    batched_record: Vec<Record>,
}

pub struct Record(String, Vec<u8>);

pub struct JobCompleteSign;

impl ThreadPool {
    pub fn new(
        num_workers: usize,
        batch_size: usize,
        channel_capacity: usize,
        input_fasta_file: &PathBuf,
        aligner: Aligner,
    ) -> Self {
        let mut workers = Vec::with_capacity(num_workers);

        let (job_sender, job_receiver) = mpsc::sync_channel(channel_capacity);
        let job_receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::new(Mutex::new(job_receiver));
        let (res_sender, res_receiver) = mpsc::channel();

        // Spawn threads
        for id in 0..(num_workers-1) {
            workers.push(Worker::new(id, aligner.clone(), job_receiver.clone(), res_sender.clone()));
        }
        workers.push(Worker::new(num_workers-1, aligner, job_receiver, res_sender));

        Self {
            workers,
            job_sender: Some(job_sender),
            res_receiver,
            input_fasta_file: input_fasta_file.clone(),
            batch_size,
        }
    }
    pub fn execute(
        &mut self,
        reference: Reference,
        reference_index: u32,
    ) -> Result<()> {
        let reference = Arc::new(reference);

        // Send jobs
        let mut fasta_reader = FastaReader::from_path(self.input_fasta_file.clone())?;
        let mut sequence_buffer = Vec::new();
        let mut label_string = String::new();

        let mut batched_record = Vec::with_capacity(self.batch_size);
        let mut remained_batch_size = self.batch_size;

        let mut total_jobs = 0;
        while let Some(mut record) = fasta_reader.next() {
            record.extend_seq_buf(&mut sequence_buffer);
            set_sequence_to_uppercase(&mut sequence_buffer);
            record.extend_id_string(&mut label_string)?;
            batched_record.push(Record(
                std::mem::replace(&mut label_string, String::new()),
                std::mem::replace(&mut sequence_buffer, Vec::new()),
            ));
            remained_batch_size -= 1;

            if remained_batch_size == 0 {
                let job = Job {
                    reference: Arc::clone(&reference),
                    reference_index,
                    batched_record: std::mem::replace(
                        &mut batched_record,
                        Vec::with_capacity(self.batch_size),
                    ),
                };
                self.job_sender.as_ref().unwrap().send(job).unwrap();
                total_jobs += 1;
                remained_batch_size = self.batch_size;
            }
        }
        if !batched_record.is_empty() { // Send last batch
            let job = Job {
                reference: Arc::clone(&reference),
                reference_index,
                batched_record,
            };
            self.job_sender.as_ref().unwrap().send(job).unwrap();
            total_jobs += 1;
        }

        // Check completion
        let mut completed_jobs = 0;
        while completed_jobs < total_jobs {
            let _ = self.res_receiver.recv().unwrap();
            completed_jobs += 1;
        }

        Ok(())
    }
    pub fn clear(&mut self) {
        drop(self.job_sender.take().unwrap());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.workers.clear()
    }
}

impl Worker {
    fn new(
        id: usize,
        mut aligner: Aligner,
        job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        res_sender: mpsc::Sender<JobCompleteSign>,
    ) -> Worker {
        eprintln!("  - Worker {} spawned", id);

        let thread = thread::spawn(move || {
            let stdout = std::io::stdout();
            let mut reference_sequence_buffer = InMemoryBuffer::new();

            loop {
                let msg = job_receiver.lock().unwrap().recv();
                let Job {
                    reference,
                    reference_index,
                    batched_record,
                } = match msg {
                    Ok(job) => job,
                    Err(_) => {
                        break;
                    },
                };

                let batched_result = {
                    let mut string_result = String::new();
                    batched_record.into_iter().for_each(|Record(label, mut query)| {
                        // Forward
                        let result = aligner.align_query_with_sequence_buffer(
                            reference.as_ref(),
                            &mut reference_sequence_buffer,
                            &query,
                        );
                        result.0.into_iter().for_each(|TargetAlignmentResult {
                            index: target_index,
                            alignments: anchor_results,
                        }| {
                            anchor_results.into_iter().for_each(|anchor_result| {
                                string_result.push_str(&format!(
                                    "{}\t1\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                                    label, reference_index, target_index, anchor_result.penalty, anchor_result.length,
                                    anchor_result.position.query.0, anchor_result.position.query.1,
                                    anchor_result.position.target.0, anchor_result.position.target.1,
                                    operations_to_string(&anchor_result.operations)
                                ));
                            });
                        });
                        
                        // Reverse complementary
                        reverse_complement_of_dna_sequence_in_place(&mut query);
                        let result = aligner.align_query_with_sequence_buffer(
                            reference.as_ref(),
                            &mut reference_sequence_buffer,
                            &query,
                        );
                        result.0.into_iter().for_each(|TargetAlignmentResult {
                            index: target_index,
                            alignments: anchor_results,
                        }| {
                            anchor_results.into_iter().for_each(|anchor_result| {
                                string_result.push_str(&format!(
                                    "{}\t0\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                                    label, reference_index, target_index, anchor_result.penalty, anchor_result.length,
                                    anchor_result.position.query.0, anchor_result.position.query.1,
                                    anchor_result.position.target.0, anchor_result.position.target.1,
                                    operations_to_string(&anchor_result.operations)
                                ));
                            });
                        });
                    });
                    string_result
                };
                let _ = stdout.lock().write_all(batched_result.as_bytes()).unwrap();
                res_sender.send(JobCompleteSign).unwrap();
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}

fn set_sequence_to_uppercase(sequence: &mut Vec<u8>) {
    sequence.make_ascii_uppercase();
}

#[inline]
fn operations_to_string(operations: &Vec<AlignmentOperations>) -> String {
    let vec = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.operation {
                AlignmentOperation::Match => 'M',
                AlignmentOperation::Subst => 'S',
                AlignmentOperation::Insertion => 'I',
                AlignmentOperation::Deletion => 'D',
            },
            op.count,
        ).into_bytes()
    }).flatten().collect();
    unsafe { String::from_utf8_unchecked(vec) }
}
