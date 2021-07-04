mod data;
mod classic_wfa;
mod dp_pairwise;

mod accuracy {
    use super::*;
    use crate::alignment::*;
    
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::sync::Mutex;
    use std::thread;
    use std::io;
    use std::io::Write;

    // reference id, sequence, option
    type Job = (String, Vec<u8>, AlignmentOption);

    #[derive(Debug, Clone)]
    struct AlignmentOption {
        score_per_length: f64,
        minimum_length: usize,
        mismatch_penalty: usize,
        gapopen_penalty: usize,
        gapext_penalty: usize,
        using_cached_wf: bool,
        get_minimum_penalty: bool,
        kmer: usize,
    }
    struct Executor {
        job_sender: mpsc::Sender<Job>,
        workers: Vec<Worker>,
    }
    impl Executor {
        fn new(pool_size: usize) -> Self {
            let (job_sender, job_receiver) = mpsc::channel();
            let job_receiver = Arc::new(Mutex::new(job_receiver));

            let mut workers = Vec::with_capacity(pool_size);

            for id in 0..pool_size {
                workers.push(
                    Worker::new(id, job_receiver.clone())
                );
            }
            Self {
                job_sender,
                workers,
            }
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
                let (ref_id, ref_seq, alignment_option) = match job_receiver.lock().expect("Job receiving err").recv() {
                    Ok(job) => {
                        job
                    }
                    Err(_) => {
                        println!("Shutting down thread {}", id);
                        break;
                    }
                };
                alignment_with_both(ref_id, ref_seq, alignment_option);
            });
            Self {
                id: id,
                thread: Some(thread),
            }
        }
    }
    fn alignment_with_both(ref_id: String, ref_seq: Vec<u8>, alignment_option: AlignmentOption) {
        let mut qry_records = data::qry_fasta_records();

        let dop_aligner = Aligner::new(
            alignment_option.score_per_length,
            alignment_option.minimum_length,
            alignment_option.mismatch_penalty,
            alignment_option.gapopen_penalty,
            alignment_option.gapext_penalty,
            alignment_option.using_cached_wf,
            alignment_option.get_minimum_penalty
        );
        while let Some(Ok(qry_record)) = qry_records.next() {
            // get result
            let dop_res = dop_aligner.perform_with_sequence(
                &ref_seq, qry_record.seq()
            );
            let dp_res = dp_pairwise::alignment(
                &ref_seq, qry_record.seq(),
                alignment_option.score_per_length,
                alignment_option.minimum_length,
                alignment_option.kmer,
                alignment_option.mismatch_penalty,
                alignment_option.gapopen_penalty,
                alignment_option.gapext_penalty
            );
            let no_res = if let None = dop_res {
                if dp_res.len() == 0 {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            // write to stdout
            if !no_res {
                let stdout = io::stdout();
                let _ = writeln!(
                    &mut stdout.lock(),
                    "# ref id: {}, qry id: {}\n{:?}\n{:?}",
                    ref_id,
                    qry_record.id(),
                    dop_res,
                    dp_res
                );
            }
        }
    }

    #[test]
    fn test_two_files() {
        // thread counts
        let thread_pool: usize = 6;
        // Alignment options
        let score_per_length: f64 = 0.1;
        let minimum_length: usize = 100;
        let mismatch_penalty: usize = 4;
        let gapopen_penalty: usize = 3;
        let gapext_penalty: usize = 2;
        let using_cached_wf: bool = true;
        let get_minimum_penalty: bool = true;
        let kmer = dp_pairwise::get_kmer(score_per_length, minimum_length, mismatch_penalty, gapopen_penalty, gapext_penalty);
        println!("# kmer: {:?}",kmer);
        let alignment_option = AlignmentOption {
            score_per_length,
            minimum_length,
            mismatch_penalty,
            gapopen_penalty,
            gapext_penalty,
            using_cached_wf,
            get_minimum_penalty,
            kmer,
        };
        // get multi thread executor
        let executor = Executor::new(thread_pool);
        let job_sender = &executor.job_sender;
        // get ref fasta records
        let mut ref_records = data::ref_fasta_records();
        while let Some(Ok(ref_record)) = ref_records.next() {
            let _ = job_sender.send((
                ref_record.id().to_string(),
                ref_record.seq().to_vec(),
                alignment_option.clone(),
            ));
        };
    }
}

mod performance {

}