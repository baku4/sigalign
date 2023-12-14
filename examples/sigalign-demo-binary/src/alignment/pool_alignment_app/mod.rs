use super::Result;
use std::{
    path::PathBuf,
    time::Instant, io::Write,
};
use clap::{
    builder::{Command, Arg},
    arg,
    ArgMatches,
    ArgAction,
    value_parser,
};

use crate::reference::{
    SigReferenceWrapper,
    ReferencePaths,
    InnerReference,
};
use sigalign::{
    aligner::{
        Aligner,
        mode::LocalMode,
        allocation_strategy::LinearStrategy,
    },
    results::{
        AlignmentResult,
        TargetAlignmentResult,
        AlignmentOperations,
        AlignmentOperation,
    },
    utils::{FastaReader, reverse_complement_of_dna},
};
type SigAligner = Aligner<LocalMode, LinearStrategy>;

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf_list: Vec<PathBuf>,
    input_reference_paths: ReferencePaths,
    // Condition
    px: u32,
    po: u32,
    pe: u32,
    min_len: u32,
    max_ppl: f32,
    // Thread pool
    thread: usize,
}

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Alignment with FASTA file (use thread pool for multiple query files)")
            .arg_required_else_help(true)
            .arg(arg!(-i --input "Input query FASTA path").display_order(1)
                .value_names(["FILE"])
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .num_args(1..)
                .required(true))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file").display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-t --thread <INT> "The number of thread").display_order(3)
                .value_parser(value_parser!(usize))
                .required(false))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true)
                .display_order(4))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum aligned length and maximum penalty per length")
                .required(true)
                .display_order(5))
    }
    pub fn run(matches: &ArgMatches) {
        let total_start = Instant::now();

        let config = {
            eprintln!("# 1. Parsing configuration");
            let start = Instant::now();
            let config = AlignmentConfig::from_matches(matches).unwrap();
            eprintln!("{:#?}", config);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
            config
        };

        let aligner = {
            eprintln!("# 2. Make aligner");
            let start = Instant::now();
            let aligner = config.make_aligner();
            eprintln!("{:#?}", aligner);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());

            aligner
        };

        {
            eprintln!("# 3. Perform alignment");
            let start = Instant::now();
            config.perform_alignment(aligner);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
        };
        
        eprintln!("# 4. All processes are completed");
        eprintln!(" - Total time elapsed: {} s", total_start.elapsed().as_secs_f64());
    }
}

impl AlignmentConfig {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // (1) Path
        let input_fasta_pathbuf_list = {
            let iterator: clap::parser::ValuesRef<_> = matches.get_many::<PathBuf>("input").expect("Invalid input fasta");
            let vec = iterator.map(|v| v.clone()).collect();
            vec
        };
        let input_reference_pathbuf = {
            matches.get_one::<PathBuf>("reference").expect("Invalid reference file").clone()
        };
        let input_reference_paths = ReferencePaths::new_for_load(&input_reference_pathbuf);

        // (2) Condition
        let (px, po, pe) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("penalties").unwrap();
            let px: u32 = iterator.next().unwrap().parse().expect("Mismatch penalty allows integer.");
            let po: u32 = iterator.next().unwrap().parse().expect("Gap-open penalty allows integer.");
            let pe: u32 = iterator.next().unwrap().parse().expect("Gap-extend penalty allows integer.");

            (px, po, pe)
        };
        let (min_len, max_ppl) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("cutoffs").unwrap();
            let min_len: u32 = iterator.next().unwrap().parse().expect("Cutoff of MINLEN allows integer.");
            let max_ppl: f32 = iterator.next().unwrap().parse().expect("Cutoff of MAXPPL allows float.");

            (min_len, max_ppl)
        };
        
        // (4) Thread
        let thread = match matches.get_one::<usize>("thread") {
            Some(v) => *v,
            None => 1,
        };

        Ok(
            Self {
                input_fasta_pathbuf_list,
                input_reference_paths,
                px,
                po,
                pe,
                min_len,
                max_ppl,
                thread,
            }
        )
    }
    fn make_aligner(&self) -> SigAligner {
        SigAligner::new(
            self.px,
            self.po,
            self.pe,
            self.min_len,
            self.max_ppl,
        ).unwrap()
    }
    // TSV line format:
    // | query label | reference index | record index | penalty | length |
    //  query start position | query end position | record start position | record end position |
    //  string operations |
    fn perform_alignment(&self, aligner: SigAligner) {
        let pool_num = self.thread;
        let mut pool = ThreadPool::new(pool_num, aligner, &self.input_fasta_pathbuf_list);

        self.input_reference_paths.0.iter().enumerate().for_each(|(ref_idx, ref_file_path)| {
            eprintln!("  Reference {}", ref_idx);
            let reference = {
                let start = Instant::now();
                let reference = SigReferenceWrapper::load_from_file(ref_file_path).unwrap();
                eprintln!("   - Load reference {} s", start.elapsed().as_secs_f64());
                reference.inner
            };

            pool.execute(ref_idx, reference);
        });

        pool.clear();
    }
}

use std::thread;
use std::sync::{mpsc, Mutex, Arc};

struct ThreadPool<'a> {
    workers: Vec<Worker>,
    job_sender: Option<mpsc::Sender<Job>>,
    res_receiver: mpsc::Receiver<JobCompleteSign>,
    input_fasta_pathbuf_list: &'a Vec<PathBuf>,
}

impl<'a> ThreadPool<'a> {
    fn new(size: usize, aligner: SigAligner, input_fasta_pathbuf_list: &'a Vec<PathBuf>) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (job_sender, job_receiver) = mpsc::channel();
        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let (res_sender, res_receiver) = mpsc::channel();

        for id in 0..(size-1) {
            workers.push(Worker::new(id, aligner.clone(), job_receiver.clone(), res_sender.clone()));
        }
        workers.push(Worker::new(size-1, aligner, job_receiver, res_sender));

        Self {
            workers,
            job_sender: Some(job_sender),
            res_receiver,
            input_fasta_pathbuf_list,
        }
    }
    fn execute(
        &mut self,
        ref_index: usize,
        reference: InnerReference,
    ) {
        let reference = Arc::new(reference);
        for input_fasta in self.input_fasta_pathbuf_list {
            let job = Job {
                ref_idx: ref_index,
                input_fasta: input_fasta.clone(),
                reference: Arc::clone(&reference),
            };
            self.job_sender.as_ref().unwrap().send(job).unwrap();
        }
        let mut complete_count = 0;
        while complete_count < self.input_fasta_pathbuf_list.len() {
            let _ = self.res_receiver.recv().unwrap();
            complete_count += 1;
        }
    }
    fn clear(&mut self) {
        drop(self.job_sender.take().unwrap());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl<'a> Drop for ThreadPool<'a> {
    fn drop(&mut self) {
        self.workers.clear()
    }
}

struct Job {
    input_fasta: PathBuf,
    ref_idx: usize,
    reference: Arc<InnerReference>,
}

type JobCompleteSign = ();

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        mut aligner: SigAligner,
        job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        res_sender: mpsc::Sender<JobCompleteSign>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            let stdout = std::io::stdout();

            loop {
                let msg = job_receiver.lock().unwrap().recv();
                let Job { input_fasta, ref_idx, reference } = match msg {
                    Ok(job) => job,
                    Err(_) => {
                        break;
                    },
                };
                
                eprintln!("   Worker {} received job: {:?}", id, &input_fasta);
                let mut sequence_buffer = reference.get_sequence_buffer();
                let fasta_reader = FastaReader::from_path(input_fasta).unwrap();
                fasta_reader.for_each(|(label, query)| {
                    // (1) Original Query
                    {
                        let result = aligner.alignment(reference.as_ref(), &mut sequence_buffer, &query);
                        result.0.into_iter().for_each(|TargetAlignmentResult {
                            index: record_index,
                            alignments: anchor_results,
                        }| {
                            anchor_results.into_iter().for_each(|anchor_result| {
                                let line = format!(
                                    "{}\t1\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                                    label, ref_idx, record_index, anchor_result.penalty, anchor_result.length,
                                    anchor_result.position.query.0, anchor_result.position.query.1,
                                    anchor_result.position.target.0, anchor_result.position.target.1,
                                    operations_to_string(&anchor_result.operations)
                                );
                
                                let _ = stdout.lock().write(line.as_bytes()).unwrap();
                            });
                        });
                    }
                    // (2) Reverse complementary Query
                    {
                        let rev_com_query = reverse_complement_of_dna(&query);
                        let result = aligner.alignment(reference.as_ref(), &mut sequence_buffer, &rev_com_query);
        
                        result.0.into_iter().for_each(|TargetAlignmentResult {
                            index: record_index,
                            alignments: anchor_results,
                        }| {
                            anchor_results.into_iter().for_each(|anchor_result| {
                                let line = format!(
                                    "{}\t0\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                                    label, ref_idx, record_index, anchor_result.penalty, anchor_result.length,
                                    anchor_result.position.query.0, anchor_result.position.query.1,
                                    anchor_result.position.target.0, anchor_result.position.target.1,
                                    operations_to_string(&anchor_result.operations)
                                );
                
                                let _ = stdout.lock().write(line.as_bytes()).unwrap();
                            });
                        });
                    }
                    res_sender.send(()).unwrap();
                });
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
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
