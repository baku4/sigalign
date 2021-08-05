mod data;
mod classic_wfa;
mod dp_pairwise;
mod multi_thread;

use crate::*;

#[derive(Debug, Clone, Copy)]
struct AlignmentOption {
    // cutoff
    score_per_length: f64,
    minimum_length: SequenceLength,
    // penalty
    mismatch_penalty: Penalty,
    gapopen_penalty: Penalty,
    gapext_penalty: Penalty,
    // other options for dropout pairwise
    using_cached_wf: bool,
    get_minimum_penalty: bool,
}

fn alignment_using_dp(
    alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>
) -> Option<dp_pairwise::DpResult> {
    let aligner = dp_pairwise::DpAligner::new(alignment_option.score_per_length, alignment_option.minimum_length, alignment_option.mismatch_penalty, alignment_option.gapopen_penalty, alignment_option.gapext_penalty);
    let res = dp_pairwise::alignment(&aligner, &ref_seq, &qry_seq);
    if res.len() == 0 {
        None
    } else {
        Some(res)
    }
}

fn alignment_using_dwfa_new_anchor(
    alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>
) -> Option<alignment::AlignmentResult> {
    let aligner = alignment::Aligner::new(alignment_option.score_per_length, alignment_option.minimum_length, alignment_option.mismatch_penalty, alignment_option.gapopen_penalty, alignment_option.gapext_penalty, alignment_option.using_cached_wf, alignment_option.get_minimum_penalty);
    let res= aligner.perform_with_sequence_using_new_anchor(&ref_seq, &qry_seq);
    res
}

mod compare_result {
    use super::*;
    use crate::*;

    #[allow(dead_code)]
    #[test]
    fn test_alignment_result() {
        // setting option
        let alignment_option = AlignmentOption {
            score_per_length: 0.1,
            minimum_length: 100,
            mismatch_penalty: 4,
            gapopen_penalty: 3,
            gapext_penalty: 2,
            using_cached_wf: true,
            get_minimum_penalty: true,
        };
        // executor
        let executor = multi_thread::print_output::Executor::new(1);
        // get 
        let map = data::get_connected_map();
        // read data
        let mut ref_records = data::ref_fasta_records();
        while let Some(Ok(ref_record)) = ref_records.next() {
            let matched_qry_id = if let Some(qry_string) = map.get(ref_record.id()) {
                qry_string.clone()
            } else {
                continue;
            };
            let mut qry_records = data::qry_fasta_records();
            while let Some(Ok(qry_record)) = qry_records.next() {
                if matched_qry_id != qry_record.id() {
                    continue;
                }
                // setting job 1
                let print_using_dp = |label: String| {
                    move |alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>| {
                        let res = alignment_using_dp(alignment_option, ref_seq, qry_seq);
                        if let Some(v) = res {
                            println!("{}\n{:?}", label, v);
                        }
                    }
                };
                let boxed_function_dp: Box::<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static> = Box::new(print_using_dp(format!(
                    "#DP,{},{}", ref_record.id().to_string(), qry_record.id().to_string()
                )));
                let job_1: multi_thread::print_output::Job = (
                    alignment_option.clone(),
                    ref_record.seq().to_vec(),
                    qry_record.seq().to_vec(),
                    boxed_function_dp,
                );
                // setting job 2
                let print_using_dropout = |label: String| {
                    move |alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>| {
                        let res = alignment_using_dwfa_new_anchor(alignment_option, ref_seq, qry_seq);
                        if let Some(v) = res {
                            println!("{}\n{:?}", label, v);
                        }
                    }
                };
                let boxed_function_dropout: Box::<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static> = Box::new(print_using_dropout(format!(
                    "#DROPOUT,{},{}", ref_record.id().to_string(), qry_record.id().to_string()
                )));
                let job_2: multi_thread::print_output::Job = (
                    alignment_option.clone(),
                    ref_record.seq().to_vec(),
                    qry_record.seq().to_vec(),
                    boxed_function_dropout,
                );
                // Send job
                let _ = executor.job_sender.send(job_1);
                let _ = executor.job_sender.send(job_2);
            }
        }
    }
}

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
                // alignment_with_both(ref_id, ref_seq, alignment_option);
            });
            Self {
                id: id,
                thread: Some(thread),
            }
        }
    }

    /*
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
    */
}