use super::*;

pub fn generate_all_semi_global_answer_using_multiple_thread(
    thread_count: usize,
) {
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
        ref_file: PathBuf,
    ) -> Self {
        let thread = thread::spawn(move || loop {
            let job  = rx.lock().unwrap().recv();
            match job {
                Ok((label, query)) => {
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
                },
                Err(_) => {
                    break;
                },
            }
        });

        Worker { thread }
    }
}
