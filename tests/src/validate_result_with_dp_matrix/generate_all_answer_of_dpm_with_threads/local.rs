use super::*;

pub fn generate_all_local_answer_using_multiple_thread(
    thread_count: usize,
) {
    init_logger();
    info!("Start to generate local result with DP matrix using threads {}", thread_count);
    
    // Prepare data
    let ref_file = get_ref_for_val_path();
    let sig_reference = Arc::new(
        DefaultReference::from_fasta_file(ref_file).unwrap()
    );

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
            Arc::clone(&sig_reference),
        ));
    }

    // Perform alignment
    let qry_reader = FastaReader::from_path(qry_file).unwrap();
    for (qry_index, (label, query)) in qry_reader.into_iter().enumerate() {
        tx.send((label, query)).unwrap();
    }
    drop(tx);
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
        sig_reference: Arc<DefaultReference>,
    ) -> Self {
        let thread = thread::spawn(move || loop {
            if let Ok((label, query)) = rx.lock().unwrap().recv() {
                info!(" - query label: {}", label);
                let _ = get_cached_local_all_substring_to_pattern_matched_targets_result_with_dp_matrix(
                    &query,
                    &label,
                    &sig_reference,
                    ALIGNER_OPTION.0,
                    ALIGNER_OPTION.1,
                    ALIGNER_OPTION.2,
                    ALIGNER_OPTION.3,
                    ALIGNER_OPTION.4,
                );
            } else {
                break;
            }
        });

        Worker { thread }
    }
}
