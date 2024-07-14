use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::sync::{mpsc, Mutex, Arc};
use anyhow::{Result, bail};
use log::info;

use sigalign::results::QueryAlignment;
use sigalign::ReferenceBuilder;
use sigalign_utils::sequence_reader::{
    SeqRecord as _,
    fasta::FastaReader,
};
use crate::common::{
    dynamic_programming_matrix::{
        dp_local_to_pattern_existing_targets,
        dp_semi_global_to_pattern_existing_targets,
        dp_local_to_ref_file,
        dp_semi_global_to_ref_file,
    },
    directory_path::get_target_dir,
    test_data::DataForValidation,
    tsv_results::{
        write_query_alignment_as_tsv_format,
        read_query_alignments_from_tsv_format,
    },
};

use super::{
    DpmTestUnit,
    DpmTestMode,
};

impl DpmTestUnit{
    // Generate Results
    pub fn save_results_to_cache_with_multiple_threads(
        &self,
        thread_num: usize,
        overwrite: bool,
    ) -> Result<()> {
        if !overwrite && self.result_cache_file.exists() {
            bail!("Result cache file already exists: {:?}", self.result_cache_file);
        }

        let tmp_file_path = self.get_tmp_file_for_cached_results();
        let tmp_file = std::fs::File::create(&tmp_file_path)?;

        let buf_writer = std::io::BufWriter::new(tmp_file);
        let locked_writer = Arc::new(Mutex::new(buf_writer));

        // Create channels
        let (job_sender, job_receiver) = mpsc::sync_channel(30);
        let job_receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::new(Mutex::new(job_receiver));
        let (res_sender, res_receiver) = mpsc::channel();

        // Create workers
        let mut workers = Vec::with_capacity(thread_num);
        for id in 0..thread_num {
            workers.push(Worker::new(
                id,
                locked_writer.clone(),
                job_receiver.clone(),
                res_sender.clone(),
            ));
        }

        // Send jobs
        let (ref_file, qry_file) = self.test_data.get_data_paths();
        let mut qry_reader = FastaReader::from_path(qry_file)?;
        let mut qry_index = 0;
        while let Some(mut record) = qry_reader.next() {
            let mut query = Vec::new();
            record.extend_seq_buf(&mut query);

            let job = Job {
                query_index: qry_index,
                query,
                ref_file: ref_file.clone(),
                mode: self.mode.clone(),
                px: self.px,
                po: self.po,
                pe: self.pe,
                minl: self.minl,
                maxp: self.maxp,
            };
            job_sender.send(job).unwrap();

            qry_index += 1;
        }

        // Wait for all jobs to be completed
        for _ in 0..qry_index {
            res_receiver.recv().unwrap();
        }

        // Change tmp file to the result cache file
        std::fs::rename(tmp_file_path, &self.result_cache_file)?;

        Ok(())
    }
    pub fn load_results_from_cache(
        &self,
    ) -> Result<Vec<Option<QueryAlignment>>> {
        let (_, qry_file) = self.test_data.get_data_paths();
        let mut qry_reader = FastaReader::from_path(qry_file)?;
        let mut qry_count = 0;
        while let Some(_) = qry_reader.next() {
            qry_count += 1; 
        }

        let mut reader = std::io::BufReader::new(std::fs::File::open(&self.result_cache_file)?);
        let results = read_query_alignments_from_tsv_format(
            qry_count, &mut reader
        )?;

        Ok(results)
    }
    fn get_tmp_file_for_cached_results(&self) -> PathBuf {
        let mut tmp_file = self.result_cache_file.clone();
        tmp_file.set_extension("tmp");
        tmp_file
    }
}

struct Job {
    query_index: u32,
    query: Vec<u8>,
    ref_file: PathBuf,
    mode: DpmTestMode,
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
}
struct JobCompleteSign;

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(
        id: usize,
        locked_writer: Arc<Mutex<std::io::BufWriter<std::fs::File>>>,
        job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        res_sender: mpsc::Sender<JobCompleteSign>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = match job_receiver.lock() {
                    Ok(receiver) => {
                        match receiver.recv() {
                            Ok(job) => job,
                            Err(_) => break,
                        }
                    },
                    Err(_) => break,
                };
                if job.query_index % 100 == 0 {
                    info!("Worker {}: Processing query index {}", id, job.query_index);
                }
                // Perform the job
                let result = match job.mode {
                    DpmTestMode::LocalWithOneMat => {
                        dp_local_to_ref_file(
                            &job.query,
                            &job.ref_file,
                            job.px,
                            job.po,
                            job.pe,
                            job.minl,
                            job.maxp,
                        )
                    },
                    DpmTestMode::SemiGlobal => {
                        dp_semi_global_to_ref_file(
                            &job.query,
                            &job.ref_file,
                            job.px,
                            job.po,
                            job.pe,
                            job.minl,
                            job.maxp,
                        )
                    },
                };

                // Write the result to the file
                let mut writer = locked_writer.lock().unwrap();
                write_query_alignment_as_tsv_format(
                    job.query_index,
                    &result,
                    &mut *writer,
                ).unwrap();

                res_sender.send(JobCompleteSign).unwrap();
            }
        });
        Self {
            thread: Some(thread),
        }
    }
}