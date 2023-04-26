use crate::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};
use ahash::AHashSet;
use sigalign::{
    wrapper::{
        DefaultAligner, DefaultReference,
    }, utils::FastaReader, results::{AlignmentResult, AnchorAlignmentResult},
};
use log::{info, error};
use super::{
    ALIGNER_OPTION,
    get_cached_semi_global_result_with_dp_matrix,
    generate_answer_with_dp_matrix,
};
use std::{
    sync::{mpsc, Arc, Mutex},
    path::PathBuf,
};
use std::thread;

#[test]
fn generate_all_answers_with_dp_matrix_using_multiple_thread() {
    let thread_count = 4;
    
    init_logger();
    info!("Start to generate semi-global result with DP matrix using threads {}", thread_count);
    
    // Prepare data
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    let (tx, rx): (
        mpsc::Sender<(String, Vec<u8>)>,
        mpsc::Receiver<(String, Vec<u8>)>,
    ) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));

    let mut workers = Vec::with_capacity(thread_count);

    for _ in 0..thread_count {
        workers.push(Worker::new(
            Arc::clone(&rx),
            ref_file.clone(),
        ));
    }

    // Perform alignment
    let qry_reader = FastaReader::from_path(qry_file).unwrap();
    for (qry_index, (label, query)) in qry_reader.into_iter().enumerate() {
        tx.send((label, query)).unwrap();
    }

    // Wait
    for worker in workers {
        worker.thread.join().unwrap();
    }
}

struct Worker {
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(
        rx: Arc<Mutex<mpsc::Receiver<(String, Vec<u8>)>>>,
        ref_file: PathBuf,
    ) -> Self {
        let thread = thread::spawn(move || loop {
            let (label, query) = rx.lock().unwrap().recv().unwrap();
            info!(" - query label: {}", label);
            let _ = get_cached_semi_global_result_with_dp_matrix(
                &query,
                &label,
                &ref_file,
                ALIGNER_OPTION.0,
                ALIGNER_OPTION.1,
                ALIGNER_OPTION.2,
                ALIGNER_OPTION.3,
                ALIGNER_OPTION.4,
            );
        });

        Worker { thread }
    }
}
